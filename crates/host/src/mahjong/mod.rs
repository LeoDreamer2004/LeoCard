use std::collections::HashSet;
use std::time::Duration;

use leocard_mahjong::{
    ActionOutcome, GameError, GameState, HandResult, MahjongClaim, MahjongClaimOption,
    MahjongMeldKind, MahjongPlayerId, MahjongRuleSet, MahjongTile, MahjongTileKind, Phase,
    build_deck,
};
use leocard_protocol::{
    ClientCommand, ClientMessage, GameCommand, GameEvent, GameKind, GameRules, GameSnapshot,
    GameViolation, LobbySnapshot, MahjongCommand, MahjongDiscardView, MahjongEvent,
    MahjongHandResultView, MahjongPendingClaimView, MahjongPhaseView, MahjongPlayerState,
    MahjongPublicMeldView, MahjongSnapshot, MahjongViolation, MahjongWinView, MatchId, PlayerId,
    PlayerInteraction, PlayerInteractionKind, RejectReason, RequestId, Revision, RoomId,
    ServerEvent,
};

use crate::{
    AUTO_PLAY_DELAY, AutoPlayDelayState, ConnectionId, Delivery, HostError, RoomSession,
    new_match_id,
};

mod automation;
mod commands;
mod lifecycle;
mod snapshot;

#[cfg(test)]
mod tests;

const MAHJONG_DEAL_INTERVAL: Duration = Duration::from_millis(320);

#[derive(Clone, Copy, Debug)]
enum AutomaticMahjongAction {
    Discard(MahjongTile),
    Respond(MahjongClaim),
    SelfDraw,
    ConcealedKong(MahjongTileKind),
    AddedKong(MahjongTile),
}

#[derive(Clone, Debug)]
pub struct MahjongSession {
    room: RoomSession,
    rules: MahjongRuleSet,
    shuffled_deck: Option<Vec<MahjongTile>>,
    game: Option<GameState>,
    match_id: Option<MatchId>,
    auto_play_delay: Option<AutoPlayDelayState>,
    deal_delay: Duration,
}

fn automatic_mahjong_action(
    game: &GameState,
    player: MahjongPlayerId,
) -> Option<AutomaticMahjongAction> {
    match game.phase() {
        Phase::WaitingForClaims(pending) => {
            let options = pending.options_for(player)?;
            if options.contains(&MahjongClaimOption::Win)
                && game.legal_claim_win_available(player).ok()?
            {
                return Some(AutomaticMahjongAction::Respond(MahjongClaim::Win));
            }
            let claim = if options.contains(&MahjongClaimOption::Kong) {
                MahjongClaim::Kong
            } else if options.contains(&MahjongClaimOption::Pung) {
                MahjongClaim::Pung
            } else if let Some(start) = options.iter().find_map(|option| match option {
                MahjongClaimOption::Chow { start } => Some(*start),
                _ => None,
            }) {
                MahjongClaim::Chow { start }
            } else {
                MahjongClaim::Pass
            };
            Some(AutomaticMahjongAction::Respond(claim))
        }
        Phase::Playing if game.current_player() == player => {
            if game.legal_self_draw_available(player).ok()? {
                return Some(AutomaticMahjongAction::SelfDraw);
            }
            let state = game.player(player)?;
            if game.wall_len() > 0
                && let Some(kind) = state.hand().iter().map(|tile| tile.kind()).find(|kind| {
                    state
                        .hand()
                        .iter()
                        .filter(|tile| tile.kind() == *kind)
                        .count()
                        == 4
                })
            {
                return Some(AutomaticMahjongAction::ConcealedKong(kind));
            }
            if game.wall_len() > 0
                && let Some(tile) = state.hand().iter().copied().find(|tile| {
                    state.melds().iter().any(|meld| {
                        meld.kind() == MahjongMeldKind::Pung && meld.tile() == tile.kind()
                    })
                })
            {
                return Some(AutomaticMahjongAction::AddedKong(tile));
            }
            automatic_discard(state.hand()).map(AutomaticMahjongAction::Discard)
        }
        Phase::Dealing { .. }
        | Phase::ReplacingFlower { .. }
        | Phase::Playing
        | Phase::Finished(_) => None,
    }
}

fn automatic_discard(hand: &[MahjongTile]) -> Option<MahjongTile> {
    hand.iter().copied().min_by_key(|tile| {
        let kind = tile.kind();
        let identical = hand
            .iter()
            .filter(|candidate| candidate.kind() == kind)
            .count()
            .saturating_sub(1) as u16;
        let neighbors = match kind {
            MahjongTileKind::Suited { suit, rank } => hand
                .iter()
                .filter_map(|candidate| match candidate.kind() {
                    MahjongTileKind::Suited {
                        suit: candidate_suit,
                        rank: candidate_rank,
                    } if candidate_suit == suit => Some(rank.abs_diff(candidate_rank)),
                    _ => None,
                })
                .map(|distance| match distance {
                    1 => 4,
                    2 => 2,
                    _ => 0,
                })
                .sum::<u16>(),
            MahjongTileKind::Wind(_) | MahjongTileKind::Dragon(_) | MahjongTileKind::Flower(_) => 0,
        };
        (identical * 7 + neighbors, kind, tile.copy())
    })
}

fn events_for_outcome(outcome: &ActionOutcome) -> Vec<MahjongEvent> {
    match outcome {
        ActionOutcome::Discarded { .. } | ActionOutcome::ClaimRecorded { .. } => Vec::new(),
        ActionOutcome::Claimed {
            player,
            source,
            tile,
            claim,
        } => vec![MahjongEvent::ClaimResolved {
            player: from_core_player(*player),
            source: from_core_player(*source),
            tile: *tile,
            claim: *claim,
        }],
        ActionOutcome::Drew { player, origin, .. } => vec![MahjongEvent::TileDrawn {
            player: from_core_player(*player),
            origin: *origin,
        }],
        ActionOutcome::KongDeclared {
            player,
            tile,
            added,
        } => vec![MahjongEvent::KongDeclared {
            player: from_core_player(*player),
            tile: *tile,
            added: *added,
        }],
        ActionOutcome::FalseWin { player, deltas } => vec![MahjongEvent::FalseWin {
            player: from_core_player(*player),
            deltas: *deltas,
        }],
        ActionOutcome::HandFinished(result) => vec![MahjongEvent::HandFinished {
            result: hand_result_view(result),
        }],
    }
}

fn hand_result_view(result: &HandResult) -> MahjongHandResultView {
    MahjongHandResultView {
        winners: result
            .winners
            .iter()
            .map(|winner| MahjongWinView {
                player: from_core_player(winner.player),
                from: winner.from.map(from_core_player),
                winning_tile: winner.winning_tile,
                score: winner.score.clone(),
            })
            .collect(),
        exhaustive_draw: result.exhaustive_draw,
        deltas: result.deltas,
        match_scores: result.match_scores,
        match_complete: result.match_complete,
        sequence_index: result.sequence_index,
    }
}

fn validate_deck(deck: &[MahjongTile]) -> Result<(), HostError> {
    if deck.len() != 144 {
        return Err(HostError::InvalidDeckSize {
            expected: 144,
            actual: deck.len(),
        });
    }
    let mut actual = deck.to_vec();
    actual.sort_unstable();
    let mut expected = build_deck();
    expected.sort_unstable();
    if actual != expected {
        return Err(HostError::InvalidDeckContents);
    }
    Ok(())
}

fn shuffled_deck() -> Vec<MahjongTile> {
    let mut deck = build_deck();
    fastrand::shuffle(&mut deck);
    deck
}

const fn to_core_player(player: PlayerId) -> MahjongPlayerId {
    MahjongPlayerId(player.0 as usize)
}

const fn from_core_player(player: MahjongPlayerId) -> PlayerId {
    PlayerId(player.0 as u8)
}
