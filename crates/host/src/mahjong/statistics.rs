use super::MahjongSession;
use crate::player::settle_completed_match_profiles_once;
use leocard_mahjong::mahjong_reference_deltas;
use leocard_protocol::{MAHJONG_MAJOR_FANS, MahjongEvent, MahjongProfileStats, PlayerId};

impl MahjongSession {
    pub(super) fn record_statistics(&mut self, events: &[MahjongEvent]) {
        for event in events {
            match event {
                MahjongEvent::FalseWin { player, .. } => {
                    let stats = &mut self.match_profile_stats[usize::from(player.0)];
                    stats.false_wins = stats.false_wins.saturating_add(1);
                }
                MahjongEvent::HandFinished { result } => {
                    let mut discarded_into_win = [false; 4];
                    for stats in &mut self.match_profile_stats {
                        stats.hands_played = stats.hands_played.saturating_add(1);
                        if result.exhaustive_draw {
                            stats.exhaustive_draws = stats.exhaustive_draws.saturating_add(1);
                        }
                    }
                    for winner in &result.winners {
                        let stats = &mut self.match_profile_stats[usize::from(winner.player.0)];
                        stats.wins = stats.wins.saturating_add(1);
                        stats.total_win_fan = stats
                            .total_win_fan
                            .saturating_add(u64::from(winner.score.total_points));
                        if let Some(source) = winner.from {
                            discarded_into_win[usize::from(source.0)] = true;
                        } else {
                            stats.self_draws = stats.self_draws.saturating_add(1);
                        }
                        let stats = &mut self.match_profile_stats[usize::from(winner.player.0)];
                        for fan in &winner.score.fans {
                            if let Some(index) =
                                MAHJONG_MAJOR_FANS.iter().position(|item| *item == fan.fan)
                            {
                                stats.major_fan_counts[index] = stats.major_fan_counts[index]
                                    .saturating_add(u32::from(fan.count));
                            }
                        }
                    }
                    for (index, was_source) in discarded_into_win.into_iter().enumerate() {
                        if was_source {
                            let stats = &mut self.match_profile_stats[index];
                            stats.discards_into_win = stats.discards_into_win.saturating_add(1);
                        }
                    }
                    if result.match_complete {
                        self.settle_match(result.match_scores);
                    }
                }
                _ => {}
            }
        }
    }

    fn settle_match(&mut self, scores: [i32; 4]) {
        let deltas = mahjong_reference_deltas(&scores, self.rules);
        let current = self.match_profile_stats.clone();
        let mut order = [0, 1, 2, 3];
        order.sort_by_key(|index| (std::cmp::Reverse(scores[*index]), *index));
        settle_completed_match_profiles_once(
            &mut self.finished_reference_changes,
            &mut self.room,
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
                let hand = &current[index];
                aggregate.hands_played = aggregate.hands_played.saturating_add(hand.hands_played);
                aggregate.wins = aggregate.wins.saturating_add(hand.wins);
                aggregate.self_draws = aggregate.self_draws.saturating_add(hand.self_draws);
                aggregate.discards_into_win = aggregate
                    .discards_into_win
                    .saturating_add(hand.discards_into_win);
                aggregate.exhaustive_draws = aggregate
                    .exhaustive_draws
                    .saturating_add(hand.exhaustive_draws);
                aggregate.false_wins = aggregate.false_wins.saturating_add(hand.false_wins);
                aggregate.total_win_fan =
                    aggregate.total_win_fan.saturating_add(hand.total_win_fan);
                for (total, count) in aggregate
                    .major_fan_counts
                    .iter_mut()
                    .zip(hand.major_fan_counts)
                {
                    *total = total.saturating_add(count);
                }
            },
        );
    }
}
