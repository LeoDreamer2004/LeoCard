use super::{
    UnoSession, append_finished_event, events_for_outcome, merge_uno_profile_stats,
    pending_draw_reveal_for_outcome, to_core_player,
};
use crate::lifecycle::HostedGameLifecycle;
use crate::player::settle_completed_match_profiles_once;
use crate::{ConnectionId, Delivery};
use leocard_protocol::{
    GameViolation, PlayerId, PlayerViolation, RejectReason, RequestId, UnoProfileStats,
};
use leocard_uno::{
    ActionOutcome, GameError, GameState, Phase, PlayedEffect, UnoCard, UnoChallengeResult,
    UnoColor, UnoPlayerId,
};

impl UnoSession {
    pub(super) fn perform_action<F>(
        &mut self,
        connection: ConnectionId,
        request_id: RequestId,
        played: Vec<(UnoCard, Option<UnoColor>)>,
        successful_jump_in: bool,
        action: F,
    ) -> Vec<Delivery>
    where
        F: FnOnce(&mut GameState, UnoPlayerId) -> Result<ActionOutcome, GameError>,
    {
        let Some(player) = self.room.player_id(connection) else {
            return self.room.reject(
                connection,
                request_id,
                RejectReason::Player(PlayerViolation::NotJoined),
            );
        };
        let Some(game) = self.game.as_mut() else {
            return self.room.reject(
                connection,
                request_id,
                RejectReason::Game(GameViolation::GameNotStarted),
            );
        };
        let jump_in_was_available =
            successful_jump_in && game.jump_in_card(to_core_player(player)).is_some();
        match action(game, to_core_player(player)) {
            Ok(outcome) => {
                if jump_in_was_available
                    && let Some(stats) = self.match_profile_stats.get_mut(usize::from(player.0))
                {
                    stats.successful_jump_ins = stats.successful_jump_ins.saturating_add(1);
                }
                let mut events = events_for_outcome(&outcome, &played);
                append_finished_event(&mut events, game);
                self.record_profile_outcome(&outcome);
                if !played.is_empty() {
                    self.record_jump_in_opportunities();
                }
                self.record_state_peaks();
                self.apply_finished_reference_points();
                self.reset_auto_play_delay();
                self.room.bump_revision();
                let draw_reveal = pending_draw_reveal_for_outcome(
                    &outcome,
                    self.game
                        .as_ref()
                        .expect("a successful UNO action keeps an active game"),
                );
                let mut deliveries = self.broadcast_events(std::mem::take(&mut events));
                if draw_reveal.is_some() || self.pending_draw_reveal.is_none() {
                    self.pending_draw_reveal = draw_reveal;
                }
                deliveries.extend(self.broadcast_game(Some((connection, request_id))));
                deliveries
            }
            Err(error) => self.reject_game_error(connection, request_id, &error),
        }
    }

    fn record_jump_in_opportunities(&mut self) {
        let Some(game) = self.game.as_ref() else {
            return;
        };
        for participant in &self.room.players {
            if game.jump_in_card(to_core_player(participant.id)).is_some()
                && let Some(stats) = self
                    .match_profile_stats
                    .get_mut(usize::from(participant.id.0))
            {
                stats.jump_in_opportunities = stats.jump_in_opportunities.saturating_add(1);
            }
        }
    }

    pub(super) fn record_profile_outcome(&mut self, outcome: &ActionOutcome) {
        match outcome {
            ActionOutcome::PenaltyDrawn { player, cards, .. }
            | ActionOutcome::SkipResolved { player, cards, .. } => {
                self.record_penalty_peak(*player, cards.len());
            }
            ActionOutcome::ChallengeResolved {
                challenger,
                offender,
                result,
                penalized,
                cards,
                ..
            } => {
                if let Some(stats) = self.match_profile_stats.get_mut(challenger.0) {
                    stats.challenges = stats.challenges.saturating_add(1);
                    if *result == UnoChallengeResult::Successful {
                        stats.successful_challenges = stats.successful_challenges.saturating_add(1);
                    }
                }
                if let Some(stats) = self.match_profile_stats.get_mut(offender.0) {
                    stats.challenges_received = stats.challenges_received.saturating_add(1);
                    if *result == UnoChallengeResult::Successful {
                        stats.successful_challenges_received =
                            stats.successful_challenges_received.saturating_add(1);
                    }
                }
                self.record_penalty_peak(*penalized, cards.len());
            }
            ActionOutcome::UnoCalled { player } => {
                if let Some(stats) = self.match_profile_stats.get_mut(player.0) {
                    stats.uno_calls = stats.uno_calls.saturating_add(1);
                }
            }
            ActionOutcome::UnoReported { target, cards, .. } => {
                if let Some(stats) = self.match_profile_stats.get_mut(target.0) {
                    stats.uno_penalties = stats.uno_penalties.saturating_add(1);
                }
                self.record_penalty_peak(*target, cards.len());
            }
            ActionOutcome::Played {
                effect: Some(PlayedEffect::DrawReflected { player, cards }),
                ..
            } => self.record_penalty_peak(*player, cards.len()),
            ActionOutcome::ColorChosen { .. }
            | ActionOutcome::Played { .. }
            | ActionOutcome::SwapOneCardTaken { .. }
            | ActionOutcome::SwapOneCompleted { .. }
            | ActionOutcome::HandsTraded { .. }
            | ActionOutcome::DrewCards { .. }
            | ActionOutcome::PassedAfterDraw { .. }
            | ActionOutcome::ColorRouletteResolved { .. } => {}
        }
    }

    fn record_penalty_peak(&mut self, player: UnoPlayerId, card_count: usize) {
        let card_count = u16::try_from(card_count).unwrap_or(u16::MAX);
        if let Some(stats) = self.match_profile_stats.get_mut(player.0) {
            stats.max_penalty_cards = stats.max_penalty_cards.max(card_count);
        }
    }

    pub(super) fn record_state_peaks(&mut self) {
        let Some(game) = self.game.as_ref() else {
            return;
        };
        let turn = game.turn();
        let peaks = game
            .players()
            .iter()
            .map(|player| {
                let hand_cards = u16::try_from(player.hand().len()).unwrap_or(u16::MAX);
                let stored_skips = game.skipped_turns(player.id()).unwrap_or_default();
                let pending_skips = turn
                    .filter(|turn| turn.current_player == player.id())
                    .map_or(0, |turn| turn.pending_skip);
                (hand_cards, stored_skips.saturating_add(pending_skips))
            })
            .collect::<Vec<_>>();
        for (stats, (hand_cards, skipped_turns)) in self.match_profile_stats.iter_mut().zip(peaks) {
            stats.max_hand_cards = stats.max_hand_cards.max(hand_cards);
            stats.max_skipped_turns = stats.max_skipped_turns.max(skipped_turns);
        }
    }

    pub(super) fn apply_finished_reference_points(&mut self) {
        let result = match self.game.as_ref().map(GameState::phase) {
            Some(Phase::Finished(result)) => result.clone(),
            _ => return,
        };
        let eliminated = self
            .game
            .as_ref()
            .expect("a finished UNO result belongs to a game")
            .players()
            .iter()
            .map(|player| player.eliminated())
            .collect::<Vec<_>>();
        let match_profile_stats = self.match_profile_stats.clone();
        let settlements = result
            .reference_deltas
            .iter()
            .copied()
            .enumerate()
            .map(|(index, delta)| (PlayerId(index as u8), delta))
            .collect::<Vec<_>>();
        let applied = settle_completed_match_profiles_once(
            &mut self.finished_reference_changes,
            &mut self.room,
            settlements,
            |participant, delta| {
                let index = usize::from(participant.id.0);
                let aggregate = participant
                    .game_profiles
                    .uno
                    .get_or_insert_with(UnoProfileStats::default);
                aggregate.completed_games = aggregate.completed_games.saturating_add(1);
                aggregate.total_reference_delta = aggregate
                    .total_reference_delta
                    .saturating_add(i64::from(delta));
                if !eliminated[index]
                    && let Some(score) = result.hand_scores.get(index)
                {
                    aggregate.total_remaining_score = aggregate
                        .total_remaining_score
                        .saturating_add(u64::from(*score));
                }
                if let Some(placement) = result.placements.get(index).copied()
                    && let Some(count) = aggregate
                        .placement_counts
                        .get_mut(usize::from(placement.saturating_sub(1)))
                {
                    *count = count.saturating_add(1);
                }
                if let Some(current) = match_profile_stats.get(index) {
                    merge_uno_profile_stats(aggregate, current);
                }
            },
        );
        if applied {
            self.room.prepare_rematch();
        }
    }
}
