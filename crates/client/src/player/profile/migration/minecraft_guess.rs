use leocard_protocol::MinecraftGuessProfileStats;
use serde::{Deserialize, Serialize};

/// Development archives recorded six placements before MC rooms supported eight seats.
#[derive(Default, Deserialize, Serialize)]
pub(super) struct SixSeatMinecraftGuessStats {
    pub completed_games: u32,
    pub total_reference_delta: i64,
    pub total_score: u64,
    pub placement_counts: [u32; 6],
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

impl From<SixSeatMinecraftGuessStats> for MinecraftGuessProfileStats {
    fn from(previous: SixSeatMinecraftGuessStats) -> Self {
        let mut placement_counts = [0; 8];
        placement_counts[..6].copy_from_slice(&previous.placement_counts);
        Self {
            completed_games: previous.completed_games,
            total_reference_delta: previous.total_reference_delta,
            total_score: previous.total_score,
            placement_counts,
            block_games: previous.block_games,
            block_wins: previous.block_wins,
            item_games: previous.item_games,
            item_wins: previous.item_wins,
            known_guesses: previous.known_guesses,
            non_worse_guesses: previous.non_worse_guesses,
            unknown_guesses: previous.unknown_guesses,
            unknown_enabled_guesses: previous.unknown_enabled_guesses,
            skips: previous.skips,
        }
    }
}
