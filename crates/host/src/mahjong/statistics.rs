use super::MahjongSession;
use crate::player::settle_completed_match_profiles_once;
use leocard_mahjong::mahjong_reference_deltas;
use leocard_protocol::{MAHJONG_MAJOR_FANS, MahjongEvent, MahjongProfileStats, PlayerId};

impl MahjongSession {
    pub(super) fn record_statistics(&mut self, events: &[MahjongEvent]) {
        for event in events {
            match event {
                MahjongEvent::FalseWin { player, .. } => {
                    let stats = self.player_stats(*player);
                    stats.false_wins = stats.false_wins.saturating_add(1);
                }
                MahjongEvent::HandFinished { result } => {
                    let mut discarded_into_win = [false; 4];
                    for participant in &mut self.room.players {
                        let stats = participant
                            .game_profiles
                            .mahjong
                            .get_or_insert_with(MahjongProfileStats::default);
                        stats.hands_played = stats.hands_played.saturating_add(1);
                        if result.exhaustive_draw {
                            stats.exhaustive_draws = stats.exhaustive_draws.saturating_add(1);
                        }
                    }
                    for winner in &result.winners {
                        let stats = self.player_stats(winner.player);
                        stats.wins = stats.wins.saturating_add(1);
                        stats.total_win_fan = stats
                            .total_win_fan
                            .saturating_add(u64::from(winner.score.total_points));
                        if let Some(source) = winner.from {
                            discarded_into_win[usize::from(source.0)] = true;
                        } else {
                            stats.self_draws = stats.self_draws.saturating_add(1);
                        }
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
                            let stats = self.player_stats(PlayerId(index as u8));
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

    fn player_stats(&mut self, player: PlayerId) -> &mut MahjongProfileStats {
        self.room
            .players
            .iter_mut()
            .find(|participant| participant.id == player)
            .expect("麻将玩家属于当前房间")
            .game_profiles
            .mahjong
            .get_or_insert_with(MahjongProfileStats::default)
    }

    fn settle_match(&mut self, scores: [i32; 4]) {
        let deltas = mahjong_reference_deltas(&scores, self.rules);
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
            },
        );
    }
}
