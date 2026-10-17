use super::BlackjackSession;
use leocard_game_common::ranked_awards;

impl BlackjackSession {
    pub(super) fn settle_match(&mut self) {
        let game = self.game.as_ref().expect("active game");
        if !game.match_complete() {
            return;
        }
        let scores: Vec<_> = game.players().iter().map(|player| player.stack).collect();
        let awards: &[i16] = match scores.len() {
            1 => &[0],
            2 => &[2, -2],
            3 => &[3, 0, -3],
            4 => &[4, 2, -2, -4],
            5 => &[4, 2, 0, -2, -4],
            6 => &[6, 4, 2, -2, -4, -6],
            _ => unreachable!("validated player count"),
        };
        let deltas = ranked_awards(&scores, awards).expect("matching score and award counts");
        let settlements = self.order.iter().copied().zip(deltas);
        let order = &self.order;
        self.room.settle_completed_match_profiles_once(
            &mut self.finished_reference_changes,
            settlements,
            |participant, delta| {
                let index = order
                    .iter()
                    .position(|id| *id == participant.id)
                    .expect("match participant");
                let current = &game.players()[index];
                let aggregate = participant
                    .game_profiles
                    .blackjack
                    .get_or_insert_with(Default::default);
                aggregate.completed_games = aggregate.completed_games.saturating_add(1);
                aggregate.total_reference_delta = aggregate
                    .total_reference_delta
                    .saturating_add(i64::from(delta));
                aggregate.total_chips_delta = aggregate
                    .total_chips_delta
                    .saturating_add(i64::from(current.stack) - i64::from(self.rules.initial_chips));
                let placement = scores
                    .iter()
                    .filter(|score| **score > current.stack)
                    .count();
                aggregate.placement_counts[placement] =
                    aggregate.placement_counts[placement].saturating_add(1);
                let total = &mut aggregate.statistics;
                let stats = current.statistics;
                for (total, count) in [
                    (&mut total.rounds, stats.rounds),
                    (&mut total.hands, stats.hands),
                    (&mut total.wins, stats.wins),
                    (&mut total.blackjacks, stats.blackjacks),
                    (&mut total.busts, stats.busts),
                    (&mut total.doubles, stats.doubles),
                    (&mut total.splits, stats.splits),
                ] {
                    *total = total.saturating_add(count);
                }
            },
        );
    }
}
