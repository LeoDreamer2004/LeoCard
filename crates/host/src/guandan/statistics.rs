use leocard_game_common::PlayerId;
use leocard_guandan::{GuandanGameResult, GuandanPlay, GuandanPlayKind, GuandanProfileStats};

#[derive(Clone, Debug, Default)]
pub(super) struct GuandanStatistics {
    players: [GuandanProfileStats; 4],
}

impl GuandanStatistics {
    pub(super) fn observe_play(&mut self, player: PlayerId, play: &GuandanPlay) {
        let stats = &mut self.players[player.index()];
        let counter = match play.kind {
            GuandanPlayKind::Straight => &mut stats.straight_plays,
            GuandanPlayKind::ConsecutivePairs => &mut stats.consecutive_pair_plays,
            GuandanPlayKind::ConsecutiveTriples => &mut stats.consecutive_triple_plays,
            GuandanPlayKind::Bomb(_) => &mut stats.bomb_plays,
            GuandanPlayKind::StraightFlush => &mut stats.straight_flush_plays,
            GuandanPlayKind::JokerBomb => &mut stats.joker_bomb_plays,
            _ => return,
        };
        *counter = counter.saturating_add(1);
    }

    pub(super) fn finish_hand(&mut self, result: &GuandanGameResult, level_advance: u8) {
        for (player, stats) in self.players.iter_mut().enumerate() {
            let won = player % 2 == usize::from(result.winning_team);
            stats.hands_played = stats.hands_played.saturating_add(1);
            stats.hands_won = stats.hands_won.saturating_add(u32::from(won));
            stats.double_finish_hands = stats
                .double_finish_hands
                .saturating_add(u32::from(won && result.known_positions == 2));
            if won {
                stats.levels_advanced = stats
                    .levels_advanced
                    .saturating_add(u64::from(level_advance));
            }
            if let Some(place) = result.finishing_order[..usize::from(result.known_positions)]
                .iter()
                .position(|id| id.index() == player)
            {
                stats.placement_counts[place] = stats.placement_counts[place].saturating_add(1);
            } else {
                stats.unranked_hands = stats.unranked_hands.saturating_add(1);
            }
        }
    }

    pub(super) fn merge_into(
        &self,
        player: PlayerId,
        aggregate: &mut GuandanProfileStats,
        won_match: bool,
        delta: i16,
    ) {
        let current = &self.players[player.index()];
        aggregate.completed_games = aggregate.completed_games.saturating_add(1);
        aggregate.total_reference_delta = aggregate
            .total_reference_delta
            .saturating_add(i64::from(delta));
        aggregate.match_wins = aggregate.match_wins.saturating_add(u32::from(won_match));
        for (total, count) in aggregate
            .placement_counts
            .iter_mut()
            .zip(current.placement_counts)
        {
            *total = total.saturating_add(count);
        }
        aggregate.levels_advanced = aggregate
            .levels_advanced
            .saturating_add(current.levels_advanced);
        for (total, count) in [
            (&mut aggregate.hands_played, current.hands_played),
            (&mut aggregate.hands_won, current.hands_won),
            (&mut aggregate.unranked_hands, current.unranked_hands),
            (
                &mut aggregate.double_finish_hands,
                current.double_finish_hands,
            ),
            (&mut aggregate.straight_plays, current.straight_plays),
            (
                &mut aggregate.consecutive_pair_plays,
                current.consecutive_pair_plays,
            ),
            (
                &mut aggregate.consecutive_triple_plays,
                current.consecutive_triple_plays,
            ),
            (&mut aggregate.bomb_plays, current.bomb_plays),
            (
                &mut aggregate.straight_flush_plays,
                current.straight_flush_plays,
            ),
            (&mut aggregate.joker_bomb_plays, current.joker_bomb_plays),
        ] {
            *total = total.saturating_add(count);
        }
    }
}
