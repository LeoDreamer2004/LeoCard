use leocard_mc_guess::{McGuessCategoryCount, McGuessCategoryStatistics, McGuessProfileStats};
use serde::{Deserialize, Serialize};

// Freeze both historical layouts; new categories must never alter old decoders.
macro_rules! legacy_guess_stats {
    ($name:ident, $seats:literal) => {
        #[derive(Default, Deserialize, Serialize)]
        pub(super) struct $name {
            pub completed_games: u32,
            pub total_reference_delta: i64,
            pub total_score: u64,
            pub placement_counts: [u32; $seats],
            pub block_games: u32,
            pub block_wins: u32,
            pub item_games: u32,
            pub item_wins: u32,
            pub known_guesses: u32,
            pub non_worse_guesses: u32,
            pub unknown_guesses: u32,
            pub unknown_enabled_guesses: u32,
            pub skips: u32,
        }

        impl From<$name> for McGuessProfileStats {
            fn from(previous: $name) -> Self {
                let mut placement_counts = [0; 8];
                placement_counts[..$seats].copy_from_slice(&previous.placement_counts);
                Self {
                    completed_games: previous.completed_games,
                    total_reference_delta: previous.total_reference_delta,
                    total_score: previous.total_score,
                    placement_counts,
                    categories: McGuessCategoryStatistics {
                        block: McGuessCategoryCount {
                            games: previous.block_games,
                            wins: previous.block_wins,
                        },
                        item: McGuessCategoryCount {
                            games: previous.item_games,
                            wins: previous.item_wins,
                        },
                        ..Default::default()
                    },
                    known_guesses: previous.known_guesses,
                    non_worse_guesses: previous.non_worse_guesses,
                    unknown_guesses: previous.unknown_guesses,
                    unknown_enabled_guesses: previous.unknown_enabled_guesses,
                    skips: previous.skips,
                }
            }
        }
    };
}

legacy_guess_stats!(SixSeatMcGuessStats, 6);
legacy_guess_stats!(PreviousMcGuessStats, 8);
