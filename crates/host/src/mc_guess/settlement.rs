use super::McGuessSession;
use leocard_mc_guess::{McGuessMode, McGuessProfileStats, reference_point_deltas};

impl McGuessSession {
    pub(super) fn settle_match(&mut self) {
        let state = self.match_state.as_ref().expect("active match");
        if !state.progress().is_last_game()
            || !self.game.as_ref().is_some_and(|game| game.finished())
        {
            return;
        }
        let scores: Vec<_> = state.scores().collect();
        let values: Vec<_> = scores.iter().map(|(_, score)| *score).collect();
        let ranked_rounds = match self.rules.mode {
            McGuessMode::TurnBased => self.rules.turn_based.rounds,
            McGuessMode::Race | McGuessMode::Graph => 1,
        };
        let mut deltas =
            reference_point_deltas(&values, ranked_rounds).expect("two to eight players");
        if self.rules.mode == McGuessMode::Graph {
            for delta in &mut deltas {
                *delta = *delta * i16::from(self.rules.graph.nodes) / 16;
                if self.rules.graph.round_robin {
                    *delta *= 3;
                }
            }
        }
        let statistics: Vec<_> = state
            .statistics()
            .map(|(player, stats)| (player, *stats))
            .collect();
        let settlements = scores
            .iter()
            .copied()
            .zip(deltas)
            .map(|((player, _), delta)| (player, delta));
        self.room.settle_completed_match_profiles_once(
            &mut self.finished_reference_changes,
            settlements,
            |player, delta| {
                let score = scores
                    .iter()
                    .find(|(id, _)| *id == player.id)
                    .expect("match player")
                    .1;
                let placement = scores.iter().filter(|(_, other)| *other > score).count();
                let current = statistics
                    .iter()
                    .find(|(id, _)| *id == player.id)
                    .expect("match statistics")
                    .1;
                let aggregate = player
                    .game_profiles
                    .mc_guess
                    .get_or_insert_with(McGuessProfileStats::default);
                aggregate.completed_games = aggregate.completed_games.saturating_add(1);
                aggregate.total_reference_delta = aggregate
                    .total_reference_delta
                    .saturating_add(i64::from(delta));
                aggregate.total_score = aggregate.total_score.saturating_add(u64::from(score));
                aggregate.placement_counts[placement] =
                    aggregate.placement_counts[placement].saturating_add(1);
                aggregate.categories.merge(current.categories);
                for (total, count) in [
                    (&mut aggregate.known_guesses, current.known_guesses),
                    (&mut aggregate.non_worse_guesses, current.non_worse_guesses),
                    (&mut aggregate.unknown_guesses, current.unknown_guesses),
                    (
                        &mut aggregate.unknown_enabled_guesses,
                        current.unknown_enabled_guesses,
                    ),
                    (&mut aggregate.skips, current.skips),
                ] {
                    *total = total.saturating_add(count);
                }
            },
        );
    }
}
