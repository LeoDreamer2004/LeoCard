use super::{
    UnoActionContext, UnoActionStatistics, UnoMatchStatistics, UnoPlayerStatistics,
    state::{DrawChain, FinishCandidate, TrackedPlayer},
};
use crate::{
    ActionOutcome, GameState, PendingSwap, Phase, UnoCard, UnoChallengeResult, UnoColor,
    UnoFlipSide, UnoPlayerId, UnoRuleSet,
};

impl UnoMatchStatistics {
    pub fn new(game: &GameState) -> Self {
        let players = game
            .players()
            .iter()
            .map(|state| {
                let count = state.hand().len() as u16;
                TrackedPlayer {
                    progress: UnoPlayerStatistics {
                        peak_hand: count,
                        max_penalty_cards: count.saturating_sub(u16::from(UnoRuleSet::HAND_SIZE)),
                        max_skipped_turns: game
                            .skipped_turns(state.id())
                            .unwrap_or_default()
                            .saturating_add(
                                game.turn()
                                    .filter(|turn| turn.current_player == state.id())
                                    .map_or(0, |turn| turn.pending_skip),
                            ),
                        has_drawn: count > u16::from(UnoRuleSet::HAND_SIZE),
                        has_penalty: count > u16::from(UnoRuleSet::HAND_SIZE),
                        ..Default::default()
                    },
                    ..Default::default()
                }
            })
            .collect();
        Self {
            players,
            chain: DrawChain::default(),
            color_change: None,
        }
    }

    pub fn player_statistics(&self, player: UnoPlayerId) -> Option<&UnoPlayerStatistics> {
        self.players.get(player.0).map(|player| &player.progress)
    }

    pub fn observe(
        &mut self,
        before: UnoActionContext,
        outcome: &ActionOutcome,
        played: &[(UnoCard, Option<UnoColor>)],
        jump_in: bool,
        game: &GameState,
    ) -> Vec<UnoActionStatistics> {
        let previous = self
            .players
            .iter()
            .map(|player| player.progress.clone())
            .collect::<Vec<_>>();
        self.record_plays(&before, outcome, played, jump_in, game);
        self.record_draws(&before, outcome, game);
        match outcome {
            ActionOutcome::UnoCalled { player } => {
                let tracked = &mut self.players[player.0];
                tracked.progress.uno_calls = tracked.progress.uno_calls.saturating_add(1);
                if before.turn.current_player == *player && !tracked.called_this_turn {
                    tracked.called_this_turn = true;
                    tracked.uno_call_run = tracked.uno_call_run.saturating_add(1);
                    tracked.progress.max_uno_call_run =
                        tracked.progress.max_uno_call_run.max(tracked.uno_call_run);
                }
            }
            ActionOutcome::UnoReported {
                reporter, target, ..
            } => {
                self.players[reporter.0].progress.uno_reports = self.players[reporter.0]
                    .progress
                    .uno_reports
                    .saturating_add(1);
                self.players[target.0].progress.uno_penalties = self.players[target.0]
                    .progress
                    .uno_penalties
                    .saturating_add(1);
            }
            ActionOutcome::ChallengeResolved {
                challenger,
                offender,
                result,
                ..
            } => {
                let successful = u32::from(*result == UnoChallengeResult::Successful);
                let stats = &mut self.players[challenger.0].progress;
                stats.challenges = stats.challenges.saturating_add(1);
                stats.successful_challenges =
                    stats.successful_challenges.saturating_add(successful);
                let stats = &mut self.players[offender.0].progress;
                stats.challenges_received = stats.challenges_received.saturating_add(1);
                stats.successful_challenges_received = stats
                    .successful_challenges_received
                    .saturating_add(successful);
            }
            ActionOutcome::HandsTraded { player, .. } if before.seven_swap() => {
                self.players[player.0].progress.hand_swaps =
                    self.players[player.0].progress.hand_swaps.saturating_add(1);
            }
            _ => {}
        }
        if !jump_in
            && before.turn.current_player == before.player
            && turn_ended(&before, outcome, game)
        {
            let tracked = &mut self.players[before.player.0];
            if !tracked.called_this_turn {
                tracked.uno_call_run = 0;
            }
            tracked.called_this_turn = false;
        }
        let winner = match game.phase() {
            Phase::Finished(result) => Some(result.winner),
            Phase::Playing => None,
        };
        for (tracked, state) in self.players.iter_mut().zip(game.players()) {
            let count = state.hand().len() as u16;
            tracked.progress.peak_hand = tracked.progress.peak_hand.max(count);
            tracked.progress.reached_twenty_four |= count == 24 && !state.eliminated();
            let stored = game.skipped_turns(state.id()).unwrap_or_default();
            let pending = game
                .turn()
                .filter(|turn| turn.current_player == state.id())
                .map_or(0, |turn| turn.pending_skip);
            tracked.progress.max_skipped_turns = tracked
                .progress
                .max_skipped_turns
                .max(stored.saturating_add(pending));
            if !played.is_empty() && game.jump_in_card(state.id()).is_some() {
                tracked.progress.jump_in_opportunities =
                    tracked.progress.jump_in_opportunities.saturating_add(1);
            }
            if count > 0 {
                tracked.finish = FinishCandidate::default();
            }
            if winner == Some(state.id()) {
                tracked.progress.finished_with_pair = tracked.finish.pair;
                tracked.progress.finished_without_uno = tracked.finish.forgotten;
                tracked.progress.finished_with_color_gift = tracked.finish.color_gift;
            }
        }
        self.players
            .iter()
            .zip(previous)
            .enumerate()
            .filter_map(|(index, (tracked, previous))| {
                let player = UnoPlayerId(index);
                let eliminated = game
                    .player(player)
                    .expect("recorder and game players stay aligned")
                    .eliminated();
                if tracked.progress == previous
                    && winner.is_none()
                    && eliminated == before.eliminated[index]
                {
                    return None;
                }
                Some(UnoActionStatistics {
                    player,
                    rules: *game.rules(),
                    wild_cards: tracked.progress.wild_cards - previous.wild_cards,
                    jump_ins: tracked.progress.jump_ins - previous.jump_ins,
                    hand_swaps: tracked.progress.hand_swaps - previous.hand_swaps,
                    progress: tracked.progress.clone(),
                    completed_game: winner.is_some(),
                    won: winner == Some(player),
                    eliminated,
                    dark_side: game.flip_side() == Some(UnoFlipSide::Dark),
                    all_opponents_eliminated: game
                        .players()
                        .iter()
                        .filter(|state| state.id() != player)
                        .all(|state| state.eliminated()),
                })
            })
            .collect()
    }
}

fn turn_ended(before: &UnoActionContext, outcome: &ActionOutcome, game: &GameState) -> bool {
    match outcome {
        ActionOutcome::Played { .. } => !game.pending_swap().is_some_and(|pending| matches!(pending,
            PendingSwap::SwapOneTarget { player } | PendingSwap::SwapOneGive { player, .. } |
            PendingSwap::ForceTrade { player } | PendingSwap::ChooseColor { player } | PendingSwap::SevenSwap { player }
            if player == before.player)),
        ActionOutcome::DrewCards { playable, .. } => playable.is_none(),
        ActionOutcome::PenaltyDrawn { .. } | ActionOutcome::ChallengeResolved { result: UnoChallengeResult::Failed, .. } => !game.turn().is_some_and(|turn| turn.current_player == before.player && (turn.pending_skip > 0 || turn.skipped_turns_remaining > 0)),
        ActionOutcome::PassedAfterDraw { .. } | ActionOutcome::SkipResolved { .. } |
        ActionOutcome::SwapOneCompleted { .. } | ActionOutcome::ColorRouletteResolved { .. } => true,
        ActionOutcome::HandsTraded { .. } => before.seven_swap(),
        ActionOutcome::ColorChosen { .. } => matches!(before.turn.pending_swap, Some(PendingSwap::ChooseColor { .. })),
        _ => false,
    }
}
