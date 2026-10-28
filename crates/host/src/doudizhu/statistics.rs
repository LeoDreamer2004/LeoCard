use leocard_doudizhu::{Action, GameState, Play, PlayKind};
use leocard_protocol::DoudizhuProfileStats;

/// 一整场的统计，重发和进入下一小局时保留，开始新场时重置。
#[derive(Clone, Debug, Default)]
pub(super) struct DoudizhuStatistics {
    pub(super) players: Vec<DoudizhuProfileStats>,
}

impl DoudizhuStatistics {
    pub(super) fn new(count: u8) -> Self {
        Self {
            players: vec![DoudizhuProfileStats::default(); usize::from(count)],
        }
    }

    pub(super) fn observe(&mut self, player: u8, action: &Action, game: &GameState) {
        let stats = &mut self.players[usize::from(player)];
        match action {
            Action::Bid(score) => {
                stats.bid_actions = stats.bid_actions.saturating_add(1);
                stats.total_bid_score = stats.total_bid_score.saturating_add(u64::from(*score));
            }
            Action::Play(_) => {
                let (_, play) = game
                    .table
                    .as_ref()
                    .expect("accepted play remains on the table");
                Self::observe_play(stats, play);
            }
            Action::Pass => {}
        }
    }

    pub(super) fn observe_play(stats: &mut DoudizhuProfileStats, play: &Play) {
        let cards = play.cards.len() as u16;
        match play.kind {
            PlayKind::Straight => {
                stats.straight_plays = stats.straight_plays.saturating_add(1);
                stats.longest_straight = stats.longest_straight.max(cards);
            }
            PlayKind::Pairs => {
                stats.consecutive_pair_plays = stats.consecutive_pair_plays.saturating_add(1);
                stats.longest_consecutive_pairs = stats.longest_consecutive_pairs.max(cards / 2);
            }
            PlayKind::Airplane | PlayKind::AirplaneSingles | PlayKind::AirplanePairs => {
                let group_size = match play.kind {
                    PlayKind::AirplaneSingles => 4,
                    PlayKind::AirplanePairs => 5,
                    _ => 3,
                };
                stats.airplane_plays = stats.airplane_plays.saturating_add(1);
                stats.longest_airplane = stats.longest_airplane.max(cards / group_size);
            }
            PlayKind::Bomb | PlayKind::Rocket => {
                stats.bomb_plays = stats.bomb_plays.saturating_add(1);
            }
            _ => {}
        }
    }

    pub(super) fn finish_hand(&mut self, landlord: u8, scores: &[i32]) {
        for (player, (stats, score)) in self.players.iter_mut().zip(scores).enumerate() {
            stats.hands_played = stats.hands_played.saturating_add(1);
            stats.landlord_hands = stats
                .landlord_hands
                .saturating_add(u32::from(player == usize::from(landlord)));
            stats.total_match_score = stats.total_match_score.saturating_add(i64::from(*score));
        }
    }

    pub(super) fn merge_into(&self, player: u8, aggregate: &mut DoudizhuProfileStats, delta: i16) {
        let current = &self.players[usize::from(player)];
        aggregate.completed_games = aggregate.completed_games.saturating_add(1);
        aggregate.total_reference_delta = aggregate
            .total_reference_delta
            .saturating_add(i64::from(delta));
        aggregate.total_match_score = aggregate
            .total_match_score
            .saturating_add(current.total_match_score);
        aggregate.hands_played = aggregate.hands_played.saturating_add(current.hands_played);
        aggregate.landlord_hands = aggregate
            .landlord_hands
            .saturating_add(current.landlord_hands);
        aggregate.bid_actions = aggregate.bid_actions.saturating_add(current.bid_actions);
        aggregate.total_bid_score = aggregate
            .total_bid_score
            .saturating_add(current.total_bid_score);
        aggregate.straight_plays = aggregate
            .straight_plays
            .saturating_add(current.straight_plays);
        aggregate.consecutive_pair_plays = aggregate
            .consecutive_pair_plays
            .saturating_add(current.consecutive_pair_plays);
        aggregate.airplane_plays = aggregate
            .airplane_plays
            .saturating_add(current.airplane_plays);
        aggregate.bomb_plays = aggregate.bomb_plays.saturating_add(current.bomb_plays);
        aggregate.longest_straight = aggregate.longest_straight.max(current.longest_straight);
        aggregate.longest_consecutive_pairs = aggregate
            .longest_consecutive_pairs
            .max(current.longest_consecutive_pairs);
        aggregate.longest_airplane = aggregate.longest_airplane.max(current.longest_airplane);
    }
}
