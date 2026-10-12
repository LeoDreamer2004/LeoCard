use crate::{LobbyPlayer, MatchId, PlayerId, PlayerReferenceChange, PlayerScore};
use leocard_minecraft_guess::{
    GuessActionStatistics, GuessHandStatistics, MatchProgress, MinecraftGuessRuleSet, RoundView,
    ScoreGain,
};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct MinecraftGuessSnapshot {
    pub match_id: MatchId,
    pub host_port: u16,
    pub you: PlayerId,
    pub host: PlayerId,
    pub players: Vec<LobbyPlayer>,
    pub rules: MinecraftGuessRuleSet,
    pub progress: MatchProgress,
    pub round: RoundView,
    pub guess_seconds_remaining: Option<u16>,
    pub scores: Vec<PlayerScore>,
    pub score_gains: Vec<ScoreGain>,
    pub reference_changes: Vec<PlayerReferenceChange>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum MinecraftGuessEvent {
    ActionAnalyzed {
        player: PlayerId,
        statistics: Box<GuessActionStatistics>,
    },
    HandAnalyzed {
        player: PlayerId,
        statistics: GuessHandStatistics,
    },
}

/// Long-term records from completed MC guessing matches.
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct MinecraftGuessProfileStats {
    pub completed_games: u32,
    pub total_reference_delta: i64,
    pub total_score: u64,
    pub placement_counts: [u32; MinecraftGuessRuleSet::MAX_PLAYERS as usize],
    /// All completed small games of this type, including games nobody solved.
    pub block_games: u32,
    pub block_wins: u32,
    pub item_games: u32,
    pub item_wins: u32,
    pub known_guesses: u32,
    /// Known guesses at least as close as the previous known guess; the first qualifies.
    pub non_worse_guesses: u32,
    pub unknown_guesses: u32,
    /// Accepted guesses only in games where unknown guesses are enabled; skips excluded.
    pub unknown_enabled_guesses: u32,
    pub skips: u32,
}

impl MinecraftGuessProfileStats {
    pub fn small_games(&self) -> u32 {
        self.block_games.saturating_add(self.item_games)
    }

    pub fn wins(&self) -> u32 {
        self.block_wins.saturating_add(self.item_wins)
    }

    pub fn actions(&self) -> u32 {
        self.known_guesses
            .saturating_add(self.unknown_guesses)
            .saturating_add(self.skips)
    }
}
