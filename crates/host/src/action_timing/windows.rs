//! Decision windows only: no wall-clock values, presentation progress or player metadata.
use leocard_blackjack::BlackjackPhase;
use leocard_doudizhu::DoudizhuPhase;
use leocard_game_common::PlayerId;
use leocard_guandan::GuandanPhase;
use leocard_protocol::{
    GameSnapshot, MahjongPhaseView, QiGui523PhaseView, ShengjiPhaseView, TexasPhaseView,
    UnoPendingSwapView, UnoPhaseView,
};
use leocard_shengji::ShengjiFiveTrumpCrossingStage;
use std::fmt::Debug;

#[derive(Debug, Eq, PartialEq)]
pub(super) struct PendingAction {
    /// Required actor, including bots and disconnected/auto-play participants.
    pub player: PlayerId,
    /// Opaque, process-local equality token, not a persistent serialization format.
    pub key: Vec<u8>,
}

/// Complete successful actions explicitly, even if the next window has the same key.
/// Remove absent windows (including presentation pauses); exclude automatic actions from averages.
/// Shengji grace requires merging recipient snapshots: only `you` has confirmation data.
pub(super) fn pending_actions(snapshot: &GameSnapshot) -> Vec<PendingAction> {
    match snapshot {
        GameSnapshot::QiGui523(s) if matches!(s.phase, QiGui523PhaseView::Playing) => {
            let Some(trick) = &s.trick else {
                return Vec::new();
            };
            let actor = s
                .players
                .iter()
                .find(|p| p.player.id == trick.current_player && p.hand_len > 0);
            let progress = (s.played_cards.len(), trick.leader, trick.records.len());
            windows(
                actor.map(|p| p.player.id),
                ("qigui523", s.match_id, progress),
            )
        }
        GameSnapshot::Texas(s) if matches!(s.phase, TexasPhaseView::ChoosingRunout) => windows(
            s.current_player,
            ("texas-runout", s.match_id, s.hand_number),
        ),
        GameSnapshot::Texas(s)
            if matches!(s.phase, TexasPhaseView::Betting { .. }) && s.blind_to_post.is_none() =>
        {
            let actor = s
                .players
                .iter()
                .find(|p| Some(p.player.id) == s.current_player && !p.folded && !p.all_in);
            let bets = (
                s.current_bet,
                s.minimum_raise_to,
                actor.map(|p| p.committed_street),
            );
            windows(
                actor.map(|p| p.player.id),
                ("texas", s.match_id, s.hand_number, &s.phase, bets),
            )
        }
        GameSnapshot::Shengji(s) => {
            let stage = match &s.phase {
                // Optional declarations during dealing are not required actions.
                ShengjiPhaseView::Dealing { .. } => return Vec::new(),
                ShengjiPhaseView::BiddingGrace { power_outage, .. } => 1 + u8::from(*power_outage),
                ShengjiPhaseView::Burying => 3,
                ShengjiPhaseView::BottomCopying { .. } => 4,
                ShengjiPhaseView::BottomCopyBurying { .. } => 5,
                ShengjiPhaseView::FiveTrumpCrossing { stage, .. } => 6 + *stage as u8,
                ShengjiPhaseView::Playing if s.throw_failure.is_none() => 8,
                _ => return Vec::new(),
            };
            let players = s
                .players
                .iter()
                .map(|p| p.player.id)
                .filter(|p| match &s.phase {
                    ShengjiPhaseView::BiddingGrace { you_confirmed, .. } => {
                        *p == s.you && !*you_confirmed
                    }
                    ShengjiPhaseView::Burying => Some(*p) == s.dealer,
                    ShengjiPhaseView::BottomCopying { player, .. }
                    | ShengjiPhaseView::BottomCopyBurying { player } => p == player,
                    ShengjiPhaseView::FiveTrumpCrossing {
                        stage: ShengjiFiveTrumpCrossingStage::Deciding,
                        eligible,
                        decided,
                        ..
                    } => eligible.contains(p) && !decided.contains(p),
                    ShengjiPhaseView::FiveTrumpCrossing {
                        crossing, returned, ..
                    } => crossing.contains(&PlayerId((p.0 + 2) % 4)) && !returned.contains(p),
                    // Also None during completed-trick presentation pauses.
                    ShengjiPhaseView::Playing => Some(*p) == s.current_player,
                    _ => false,
                });
            let progress = (s.bottom_copy_decisions.len(), s.played_cards.len());
            let state = (s.hand_number, stage, &s.declaration, progress);
            windows(players, ("shengji", s.match_id, state))
        }
        GameSnapshot::Uno(s) if matches!(s.phase, UnoPhaseView::Playing) => {
            // None is used during automatic draw reveals; do not time the animation.
            let Some(current) = s.current_player else {
                return Vec::new();
            };
            let player = match s.pending_swap {
                Some(
                    UnoPendingSwapView::SwapOneTarget { player }
                    | UnoPendingSwapView::SwapOneGive { player, .. }
                    | UnoPendingSwapView::ForceTrade { player }
                    | UnoPendingSwapView::ChooseColor { player }
                    | UnoPendingSwapView::SevenSwap { player }
                    | UnoPendingSwapView::ColorRoulette { player },
                ) => player,
                None => current,
            };
            let actor = s
                .players
                .iter()
                .find(|p| p.player.id == player && !p.eliminated);
            // Public hand length distinguishes drawing then choosing without depending on `you`.
            // Jump-in, UNO calls and reports are optional, not additional required actors.
            windows(
                actor.map(|p| p.player.id),
                (
                    "uno",
                    s.match_id,
                    s.pending_swap,
                    (s.discard_top, s.current_color, s.flip_side, s.direction),
                    (
                        s.pending_draw,
                        s.pending_kind,
                        s.challenge_offender,
                        s.pending_skip,
                        s.sudden_death_free_play,
                    ),
                    actor.map(|p| p.hand_len),
                ),
            )
        }
        GameSnapshot::Mahjong(s) => {
            if matches!(s.phase, MahjongPhaseView::WaitingForClaims) {
                let Some(claim) = &s.pending_claim else {
                    return Vec::new();
                };
                // Other responders leaving waiting_for must not restart this player's clock.
                let source = (claim.source, claim.tile, claim.robbing_kong);
                return windows(
                    claim.waiting_for.iter().copied(),
                    ("mahjong-claim", s.match_id, s.discards.len(), source),
                );
            }
            if !matches!(s.phase, MahjongPhaseView::Playing) {
                return Vec::new();
            }
            let actor = s.players.iter().find(|p| p.player.id == s.current_player);
            let hand = actor.map(|p| (p.concealed_count, p.melds.len()));
            windows(
                actor.map(|p| p.player.id),
                (
                    "mahjong-play",
                    s.match_id,
                    s.discards.len(),
                    s.wall_len,
                    hand,
                ),
            )
        }
        GameSnapshot::Blackjack(s)
            if !s.match_complete
                && matches!(s.phase, BlackjackPhase::Betting | BlackjackPhase::Playing) =>
        {
            // `available` belongs to `you`; it is empty when viewing another actor's turn.
            let actor = s
                .players
                .iter()
                .find(|p| Some(p.player.id) == s.current_player);
            let hand = actor.and_then(|p| s.current_hand.and_then(|i| p.hands.get(usize::from(i))));
            let cards = (actor.map(|p| p.hands.len()), hand.map(|h| h.cards.len()));
            let state = (s.round, s.phase, s.current_hand, cards);
            windows(actor.map(|p| p.player.id), ("blackjack", s.match_id, state))
        }
        GameSnapshot::Doudizhu(s)
            if !s.match_complete
                && matches!(s.phase, DoudizhuPhase::Bidding | DoudizhuPhase::Playing) =>
        {
            let state = (
                s.progress.hand_number,
                s.phase,
                &s.bids,
                s.played.len(),
                &s.table,
            );
            windows(Some(s.current), ("doudizhu", s.match_id, state))
        }
        GameSnapshot::Guandan(s)
            if !s.match_complete
                && matches!(s.phase, GuandanPhase::Tribute | GuandanPhase::Playing) =>
        {
            // Current actor is public even during tribute. Never include private tribute cards
            // or recipient-only eligibility in the equality token or timing snapshot.
            windows(
                s.current,
                (
                    "guandan",
                    s.match_id,
                    s.hand_number,
                    &s.phase,
                    s.played_cards.len(),
                    &s.table,
                    &s.last_actions,
                ),
            )
        }
        GameSnapshot::MahjongLink(s) if !s.match_complete => {
            // A row push is optional preparation within this decision, not a new window.
            windows(s.current_player, ("mahjong-link", s.match_id, s.turn))
        }
        GameSnapshot::McGuess(s) => {
            s.players
                .iter()
                .filter(|p| s.round.can_guess(p.player.id, s.rules.mode))
                .flat_map(|p| {
                    let player = p.player.id;
                    // Race guesses by other players do not create a new window for this player.
                    let guesses = s
                        .round
                        .guesses
                        .iter()
                        .filter(|g| g.player == player)
                        .count();
                    // Selecting nodes is not a guess and must not restart the clock.
                    let graph = s.graph.as_ref().map(|g| {
                        g.guesses
                            .iter()
                            .filter(|r| r.guess.player == player)
                            .count()
                    });
                    windows(
                        Some(player),
                        (
                            "mc_guess",
                            s.match_id,
                            s.progress.game_index,
                            guesses,
                            graph,
                            s.round
                                .skipped_players
                                .iter()
                                .filter(|p| **p == player)
                                .count(),
                        ),
                    )
                })
                .collect()
        }
        _ => Vec::new(),
    }
}

fn windows(players: impl IntoIterator<Item = PlayerId>, context: impl Debug) -> Vec<PendingAction> {
    players
        .into_iter()
        .map(|player| PendingAction {
            player,
            key: format!("{:?}", (player, &context)).into_bytes(),
        })
        .collect()
}
