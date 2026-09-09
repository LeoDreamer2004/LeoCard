use super::{MahjongSession, from_core_player, hand_result_view, to_core_player};
use crate::{ConnectionId, Delivery};
use leocard_mahjong::{GameError, MahjongMeldKind, Phase};
use leocard_protocol::{
    GameViolation, MahjongDiscardView, MahjongEvent, MahjongPendingClaimView, MahjongPhaseView,
    MahjongPlayerState, MahjongPublicMeldView, MahjongSnapshot, MahjongViolation, PlayerId,
    RejectReason, RequestId,
};
use std::collections::HashSet;

impl MahjongSession {
    pub(super) fn broadcast_events(&self, events: Vec<MahjongEvent>) -> Vec<Delivery> {
        self.room.broadcast_game_events(events)
    }

    pub(super) fn game_snapshot(&self, recipient: PlayerId) -> MahjongSnapshot {
        let game = self
            .game
            .as_ref()
            .expect("a Mahjong snapshot requires a game");
        let core_recipient = to_core_player(recipient);
        let players = self
            .room
            .players
            .iter()
            .filter(|player| !player.left)
            .map(|participant| {
                let metadata = participant.public_metadata();
                let core_id = to_core_player(metadata.id);
                let game_player = game
                    .public_player(core_recipient, core_id)
                    .expect("room and Mahjong players stay aligned");
                MahjongPlayerState {
                    id: metadata.id,
                    profile_id: metadata.profile_id,
                    name: metadata.name,
                    avatar: metadata.avatar,
                    seat: metadata.seat.expect("started players retain seats"),
                    seat_wind: game.seat_wind(core_id).expect("started player has a wind"),
                    concealed_count: game_player.concealed_count as u8,
                    revealed_hand: game_player.revealed_hand,
                    melds: game_player
                        .melds
                        .into_iter()
                        .map(|meld| MahjongPublicMeldView {
                            kind: meld.kind,
                            tile: meld.tile,
                            claimed_from: meld.claimed_from.map(from_core_player),
                        })
                        .collect(),
                    flowers: game_player.flowers,
                    dead_hand: game_player.dead_hand,
                    ready: metadata.ready,
                    connected: metadata.connected,
                    reference_points: metadata.reference_points,
                    completed_games: metadata.completed_games,
                    game_profiles: metadata.game_profiles,
                }
            })
            .collect();
        let own = game
            .player(core_recipient)
            .expect("room and Mahjong players stay aligned");
        let pending_claim = match game.phase() {
            Phase::WaitingForClaims(pending) => Some(MahjongPendingClaimView {
                source: from_core_player(pending.source_player()),
                tile: pending.tile(),
                robbing_kong: pending.is_robbing_kong_window(),
                your_options: pending
                    .options_for(core_recipient)
                    .unwrap_or_default()
                    .to_vec(),
                your_response: pending.response_from(core_recipient),
                waiting_for: pending
                    .waiting_for()
                    .into_iter()
                    .map(from_core_player)
                    .collect(),
            }),
            Phase::Dealing { .. }
            | Phase::ReplacingFlower { .. }
            | Phase::Playing
            | Phase::Finished(_) => None,
        };
        let can_declare_kong = matches!(game.phase(), Phase::Playing)
            && game.current_player() == core_recipient
            && game.wall_len() > 0;
        let concealed_kong_options = if can_declare_kong {
            let mut kinds = own
                .hand()
                .iter()
                .map(|tile| tile.kind())
                .collect::<Vec<_>>();
            kinds.sort_unstable();
            kinds
                .iter()
                .copied()
                .collect::<HashSet<_>>()
                .into_iter()
                .filter(|kind| kinds.iter().filter(|held| **held == *kind).count() == 4)
                .collect()
        } else {
            Vec::new()
        };
        let pung_kinds = own
            .melds()
            .iter()
            .filter(|meld| meld.kind() == MahjongMeldKind::Pung)
            .map(|meld| meld.tile())
            .collect::<HashSet<_>>();
        let added_kong_options = if can_declare_kong {
            own.hand()
                .iter()
                .copied()
                .filter(|tile| pung_kinds.contains(&tile.kind()))
                .collect()
        } else {
            Vec::new()
        };
        let phase = match game.phase() {
            Phase::Dealing { batch } => MahjongPhaseView::Dealing { batch: *batch },
            Phase::ReplacingFlower { player } => MahjongPhaseView::ReplacingFlower {
                player: from_core_player(*player),
            },
            Phase::Playing => MahjongPhaseView::Playing,
            Phase::WaitingForClaims(_) => MahjongPhaseView::WaitingForClaims,
            Phase::Finished(result) => MahjongPhaseView::Finished {
                result: hand_result_view(result),
            },
        };
        MahjongSnapshot {
            match_id: self.match_id.expect("started Mahjong game has a match id"),
            host_port: self.room.host_port,
            you: recipient,
            host: self
                .room
                .host_player_id()
                .expect("started Mahjong room retains a host"),
            rules: self.rules,
            players,
            your_hand: own.hand().to_vec(),
            your_drawn_tile: (game.current_player() == core_recipient)
                .then(|| game.last_drawn())
                .flatten(),
            discards: game
                .discards()
                .iter()
                .map(|discard| MahjongDiscardView {
                    player: from_core_player(discard.player),
                    tile: discard.tile,
                    claimed_by: discard.claimed_by.map(from_core_player),
                })
                .collect(),
            dealer: from_core_player(game.dealer()),
            prevalent_wind: game.prevalent_wind(),
            sequence_index: game.sequence_index(),
            current_player: from_core_player(game.current_player()),
            wall_len: game.wall_len() as u16,
            match_scores: *game.match_scores(),
            pending_claim,
            can_self_draw: game.self_draw_available(core_recipient).unwrap_or(false),
            concealed_kong_options,
            added_kong_options,
            phase,
        }
    }

    pub(super) fn reject_game_error(
        &self,
        connection: ConnectionId,
        request_id: RequestId,
        error: &GameError,
    ) -> Vec<Delivery> {
        let violation = match error {
            GameError::InvalidPlayer(_) => MahjongViolation::InvalidPlayer,
            GameError::NotPlayersTurn { .. } => MahjongViolation::NotPlayersTurn,
            GameError::WrongPhase => MahjongViolation::WrongPhase,
            GameError::TileNotInHand(_) => MahjongViolation::TileNotInHand,
            GameError::InvalidClaim => MahjongViolation::InvalidClaim,
            GameError::AlreadyResponded => MahjongViolation::AlreadyResponded,
            GameError::CannotWin => MahjongViolation::CannotWin,
            GameError::CannotKong => MahjongViolation::CannotKong,
            GameError::InvalidHandReplacement(reason) => {
                MahjongViolation::InvalidDeveloperHand(*reason)
            }
            GameError::InvalidRules(_)
            | GameError::InvalidDeckSize { .. }
            | GameError::InvalidDeckContents
            | GameError::Score(_) => {
                return self.room.reject(
                    connection,
                    request_id,
                    RejectReason::Game(GameViolation::InvalidRuleConfiguration),
                );
            }
        };
        self.room.reject(
            connection,
            request_id,
            RejectReason::Game(GameViolation::Mahjong(violation)),
        )
    }
}
