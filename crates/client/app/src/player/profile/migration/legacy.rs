//! Frozen game profile layouts from released archives, before per-game timing.
use leocard_blackjack::BlackjackPlayerStatistics;
use leocard_mc_guess::McGuessCategoryStatistics;
use serde::{Deserialize, Serialize};

macro_rules! legacy_profile {
    ($legacy:ident, $current:path { $($field:ident: $ty:ty,)* }) => {
        #[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
        pub(in crate::player::profile) struct $legacy {
            $(pub(super) $field: $ty,)*
        }

        impl From<$legacy> for $current {
            fn from(previous: $legacy) -> Self {
                Self {
                    $($field: previous.$field,)*
                    action_time: Default::default(),
                }
            }
        }

        #[cfg(test)]
        impl From<$current> for $legacy {
            fn from(current: $current) -> Self {
                Self { $($field: current.$field,)* }
            }
        }
    };
}

legacy_profile!(
    QiGui523ProfileStats,
    leocard_qigui523::QiGui523ProfileStats {
        completed_games: u32,
        total_score: u64,
        total_reference_delta: i64,
        placement_counts: [u32; 6],
        straight_plays: u32,
        consecutive_pair_plays: u32,
        airplane_plays: u32,
        bomb_plays: u32,
        heaven_bomb_plays: u32,
        longest_straight: u16,
        longest_consecutive_pairs: u16,
        longest_airplane: u16,
    }
);

legacy_profile!(
    TexasProfileStats,
    leocard_texas_holdem::TexasProfileStats {
        completed_games: u32,
        total_reference_delta: i64,
        total_final_chips: u64,
        placement_counts: [u32; 8],
        wagered_chips: u64,
        wager_actions: u32,
        voluntary_actions: u32,
        check_actions: u32,
        raise_actions: u32,
        all_in_actions: u32,
        hands_played: u32,
        hands_folded: u32,
        hand_category_counts: [u32; 10],
        entry_tracked_hands: u32,
        entered_hands: u32,
    }
);

legacy_profile!(
    ShengjiProfileStats,
    leocard_shengji::ShengjiProfileStats {
        completed_games: u32,
        total_reference_delta: i64,
        dealer_team_games: u32,
        dealer_team_score: u64,
        collecting_team_games: u32,
        collecting_team_score: u64,
        dealer_games: u32,
        declaration_games: u32,
        counter_games: u32,
        defended_kitty_games: u32,
        captured_kitty_games: u32,
        buried_games: u32,
        buried_points: u64,
        plays: u32,
        winning_plays: u32,
        crossing_games: u32,
        play_category_counts: [u32; 5],
        longest_tractor: u16,
        longest_titanic: u16,
        longest_spaceship: u16,
        longest_throw: u16,
    }
);

legacy_profile!(
    UnoProfileStats,
    leocard_uno::UnoProfileStats {
        completed_games: u32,
        total_reference_delta: i64,
        total_remaining_score: u64,
        placement_counts: [u32; 8],
        max_hand_cards: u16,
        max_penalty_cards: u16,
        max_skipped_turns: u16,
        uno_calls: u32,
        uno_penalties: u32,
        challenges: u32,
        successful_challenges: u32,
        challenges_received: u32,
        successful_challenges_received: u32,
        jump_in_opportunities: u32,
        successful_jump_ins: u32,
    }
);

legacy_profile!(
    MahjongProfileStats,
    leocard_mahjong::MahjongProfileStats {
        completed_games: u32,
        total_reference_delta: i64,
        total_match_score: i64,
        placement_counts: [u32; 4],
        hands_played: u32,
        wins: u32,
        self_draws: u32,
        discards_into_win: u32,
        exhaustive_draws: u32,
        false_wins: u32,
        total_win_fan: u64,
        major_fan_counts: [u32; 15],
    }
);

legacy_profile!(
    McGuessProfileStats,
    leocard_mc_guess::McGuessProfileStats {
        completed_games: u32,
        total_reference_delta: i64,
        total_score: u64,
        placement_counts: [u32; 8],
        categories: McGuessCategoryStatistics,
        known_guesses: u32,
        non_worse_guesses: u32,
        unknown_guesses: u32,
        unknown_enabled_guesses: u32,
        skips: u32,
    }
);

legacy_profile!(
    BlackjackProfileStats,
    leocard_blackjack::BlackjackProfileStats {
        completed_games: u32,
        total_reference_delta: i64,
        total_chips_delta: i64,
        placement_counts: [u32; 6],
        statistics: BlackjackPlayerStatistics,
    }
);

legacy_profile!(
    DoudizhuProfileStats,
    leocard_doudizhu::DoudizhuProfileStats {
        completed_games: u32,
        total_reference_delta: i64,
        total_match_score: i64,
        hands_played: u32,
        landlord_hands: u32,
        bid_actions: u32,
        total_bid_score: u64,
        straight_plays: u32,
        consecutive_pair_plays: u32,
        airplane_plays: u32,
        bomb_plays: u32,
        longest_straight: u16,
        longest_consecutive_pairs: u16,
        longest_airplane: u16,
    }
);

legacy_profile!(
    GuandanProfileStats,
    leocard_guandan::GuandanProfileStats {
        completed_games: u32,
        total_reference_delta: i64,
        match_wins: u32,
        hands_played: u32,
        hands_won: u32,
        placement_counts: [u32; 4],
        unranked_hands: u32,
        double_finish_hands: u32,
        levels_advanced: u64,
        straight_plays: u32,
        consecutive_pair_plays: u32,
        consecutive_triple_plays: u32,
        bomb_plays: u32,
        straight_flush_plays: u32,
        joker_bomb_plays: u32,
    }
);
