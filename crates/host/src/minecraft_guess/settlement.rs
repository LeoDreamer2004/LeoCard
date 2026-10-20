use super::MinecraftGuessSession;
use leocard_minecraft_guess::{GuessMode, reference_point_deltas};
use leocard_protocol::{MinecraftGuessProfileStats, PlayerId};

impl MinecraftGuessSession {
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
            GuessMode::TurnBased => self.rules.turn_based.rounds,
            GuessMode::Race | GuessMode::Graph => 1,
        };
        let mut deltas =
            reference_point_deltas(&values, ranked_rounds).expect("two to eight players");
        if self.rules.mode == GuessMode::Graph {
            for delta in &mut deltas {
                *delta = *delta * i16::from(self.rules.graph.nodes) / 16;
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
            .map(|((player, _), delta)| (PlayerId(player), delta));
        self.room.settle_completed_match_profiles_once(
            &mut self.finished_reference_changes,
            settlements,
            |player, delta| {
                let score = scores
                    .iter()
                    .find(|(id, _)| *id == player.id.0)
                    .expect("match player")
                    .1;
                let placement = scores.iter().filter(|(_, other)| *other > score).count();
                let current = statistics
                    .iter()
                    .find(|(id, _)| *id == player.id.0)
                    .expect("match statistics")
                    .1;
                let aggregate = player
                    .game_profiles
                    .minecraft_guess
                    .get_or_insert_with(MinecraftGuessProfileStats::default);
                aggregate.completed_games = aggregate.completed_games.saturating_add(1);
                aggregate.total_reference_delta = aggregate
                    .total_reference_delta
                    .saturating_add(i64::from(delta));
                aggregate.total_score = aggregate.total_score.saturating_add(u64::from(score));
                aggregate.placement_counts[placement] =
                    aggregate.placement_counts[placement].saturating_add(1);
                for (total, count) in [
                    (&mut aggregate.block_games, current.block_games),
                    (&mut aggregate.block_wins, current.block_wins),
                    (&mut aggregate.item_games, current.item_games),
                    (&mut aggregate.item_wins, current.item_wins),
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
