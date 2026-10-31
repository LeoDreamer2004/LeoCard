use super::{MahjongSession, hand_result_view};
use crate::{ConnectionId, Delivery};
use leocard_game_common::PlayerId;
use leocard_mahjong::{MahjongGameError, MahjongMeldKind, MahjongPhase};
use leocard_protocol::{
    GameEvent, GameViolation, MahjongEvent, MahjongPendingClaimView, MahjongPhaseView,
    MahjongPlayerView, MahjongSnapshot, MahjongViolation, RejectReason, RequestId, ServerEvent,
};
use std::collections::HashSet;

impl MahjongSession {
    pub(super) fn broadcast_events(&mut self, mut events: Vec<MahjongEvent>) -> Vec<Delivery> {
        if let Some(game) = self.game.as_ref() {
            self.win_feedback.after_action(game, &mut events);
        }
        self.room
            .broadcast_game_events(
                events,
                self.match_id.map(|id| {
                    (
                        id,
                        self.game
                            .as_ref()
                            .map(|game| u32::from(game.sequence_index())),
                    )
                }),
            )
            .into_iter()
            .filter(|delivery| match &delivery.message.event {
                ServerEvent::GameEvent(GameEvent::Mahjong(MahjongEvent::WinUnavailable {
                    player,
                    ..
                })) => self.room.player_id(delivery.recipient) == Some(*player),
                _ => true,
            })
            .collect()
    }

    pub(super) fn game_snapshot(&self, recipient: PlayerId) -> MahjongSnapshot {
        let game = self
            .game
            .as_ref()
            .expect("a Mahjong snapshot requires a game");

        let players = self
            .room
            .players
            .iter()
            .filter(|player| !player.left)
            .map(|participant| {
                let metadata = participant.public_metadata();
                let core_id = metadata.player.id;
                // Public visibility is evaluated from another player's perspective for observers.
                let viewer = if game.player(recipient).is_some() {
                    recipient
                } else {
                    game.players()
                        .iter()
                        .find(|player| player.id() != core_id)
                        .expect("four players")
                        .id()
                };
                let game_player = game
                    .public_player(viewer, core_id)
                    .expect("room and Mahjong players stay aligned");
                MahjongPlayerView {
                    seat: metadata.seat.expect("started players retain seats"),
                    seat_wind: game.seat_wind(core_id).expect("started player has a wind"),
                    concealed_count: game_player.concealed_count as u8,
                    revealed_hand: game_player.revealed_hand,
                    melds: game_player.melds,
                    flowers: game_player.flowers,
                    dead_hand: game_player.dead_hand,
                    auto_play: metadata.auto_play,
                    ready: metadata.ready,
                    connected: metadata.connected,
                    player: metadata.player,
                }
            })
            .collect();
        let own = game.player(recipient);
        let pending_claim = match game.phase() {
            MahjongPhase::WaitingForClaims(pending) => Some(MahjongPendingClaimView {
                source: pending.source_player(),
                tile: pending.tile(),
                robbing_kong: pending.is_robbing_kong_window(),
                your_options: pending.options_for(recipient).unwrap_or_default().to_vec(),
                can_legal_win: game.legal_claim_win_available(recipient).unwrap_or(false),
                your_response: pending.response_from(recipient),
                waiting_for: pending.waiting_for().into_iter().collect(),
            }),
            MahjongPhase::Dealing { .. }
            | MahjongPhase::ReplacingFlower { .. }
            | MahjongPhase::Playing
            | MahjongPhase::Finished(_) => None,
        };
        let can_declare_kong = matches!(game.phase(), MahjongPhase::Playing)
            && game.current_player() == recipient
            && game.wall_len() > 0;
        let concealed_kong_options = if can_declare_kong {
            let mut kinds = own
                .expect("current player has a hand")
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
            .into_iter()
            .flat_map(|player| player.melds().iter())
            .filter(|meld| meld.kind() == MahjongMeldKind::Pung)
            .map(|meld| meld.tile())
            .collect::<HashSet<_>>();
        let added_kong_options = if can_declare_kong {
            own.expect("current player has a hand")
                .hand()
                .iter()
                .copied()
                .filter(|tile| pung_kinds.contains(&tile.kind()))
                .collect()
        } else {
            Vec::new()
        };
        let phase = match game.phase() {
            MahjongPhase::Dealing { batch } => MahjongPhaseView::Dealing { batch: *batch },
            MahjongPhase::ReplacingFlower { player } => {
                MahjongPhaseView::ReplacingFlower { player: *player }
            }
            MahjongPhase::Playing => MahjongPhaseView::Playing,
            MahjongPhase::WaitingForClaims(_) => MahjongPhaseView::WaitingForClaims,
            MahjongPhase::Finished(result) => {
                let mut result = hand_result_view(result);
                if result.match_complete {
                    result.reference_changes =
                        self.finished_reference_changes.clone().unwrap_or_default();
                }
                MahjongPhaseView::Finished { result }
            }
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
            your_hand: own.map(|player| player.hand().to_vec()).unwrap_or_default(),
            your_drawn_tile: (game.current_player() == recipient)
                .then(|| game.last_drawn())
                .flatten(),
            discards: game.discards().to_vec(),
            dealer: game.dealer(),
            prevalent_wind: game.prevalent_wind(),
            sequence_index: game.sequence_index(),
            current_player: game.current_player(),
            wall_len: game.wall_len() as u16,
            match_scores: *game.match_scores(),
            pending_claim,
            can_self_draw: game.self_draw_available(recipient).unwrap_or(false),
            can_legal_self_draw: game.legal_self_draw_available(recipient).unwrap_or(false),
            concealed_kong_options,
            added_kong_options,
            phase,
        }
    }

    pub(super) fn reject_game_error(
        &self,
        connection: ConnectionId,
        request_id: RequestId,
        error: &MahjongGameError,
    ) -> Vec<Delivery> {
        let violation = match error {
            MahjongGameError::InvalidPlayer(_) => MahjongViolation::InvalidPlayer,
            MahjongGameError::NotPlayersTurn { .. } => MahjongViolation::NotPlayersTurn,
            MahjongGameError::WrongPhase => MahjongViolation::WrongPhase,
            MahjongGameError::TileNotInHand(_) => MahjongViolation::TileNotInHand,
            MahjongGameError::InvalidClaim => MahjongViolation::InvalidClaim,
            MahjongGameError::AlreadyResponded => MahjongViolation::AlreadyResponded,
            MahjongGameError::CannotWin => MahjongViolation::CannotWin,
            MahjongGameError::CannotKong => MahjongViolation::CannotKong,
            MahjongGameError::InvalidHandReplacement(reason) => {
                MahjongViolation::InvalidDeveloperHand(*reason)
            }
            MahjongGameError::InvalidRules(_)
            | MahjongGameError::InvalidDeckSize { .. }
            | MahjongGameError::InvalidDeckContents
            | MahjongGameError::Score(_) => {
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
