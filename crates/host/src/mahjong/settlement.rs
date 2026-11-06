use super::MahjongSession;
use crate::features::MatchSettlement;
use leocard_game_common::PlayerId;
use leocard_mahjong::{MahjongProfileStats, mahjong_reference_deltas};
use std::cmp::Reverse;

impl MatchSettlement for MahjongSession {
    type Outcome = [i32; 4];

    fn commit_match(&mut self, scores: &Self::Outcome) {
        let scores = *scores;

        let deltas = mahjong_reference_deltas(&scores, self.rules);
        let mut order = [0, 1, 2, 3];
        order.sort_by_key(|index| (Reverse(scores[*index]), *index));
        self.room.settle_completed_match_profiles_once(
            &mut self.finished_reference_changes,
            deltas
                .into_iter()
                .enumerate()
                .map(|(index, delta)| (PlayerId(index as u8), delta)),
            |player, delta| {
                let index = usize::from(player.id.0);
                let rank = 1 + order
                    .iter()
                    .position(|candidate| *candidate == index)
                    .unwrap();
                let aggregate = player
                    .game_profiles
                    .mahjong
                    .get_or_insert_with(MahjongProfileStats::default);
                aggregate.completed_games = aggregate.completed_games.saturating_add(1);
                aggregate.total_reference_delta = aggregate
                    .total_reference_delta
                    .saturating_add(i64::from(delta));
                aggregate.total_match_score = aggregate
                    .total_match_score
                    .saturating_add(i64::from(scores[index]));
                aggregate.placement_counts[rank - 1] =
                    aggregate.placement_counts[rank - 1].saturating_add(1);
            },
        );
    }
}
