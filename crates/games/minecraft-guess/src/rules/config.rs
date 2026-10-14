use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub enum GuessPool {
    Blocks,
    Items,
    #[default]
    Mixed,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct GuessOptions {
    pub pool: GuessPool,
    pub candidate_hints: bool,
    pub allow_unknown_guesses: bool,
}

impl Default for GuessOptions {
    fn default() -> Self {
        Self {
            pool: GuessPool::default(),
            candidate_hints: true,
            allow_unknown_guesses: false,
        }
    }
}

impl GuessOptions {
    pub(super) fn is_valid(self) -> bool {
        !(self.candidate_hints && self.allow_unknown_guesses)
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct TurnBasedRules {
    pub options: GuessOptions,
    pub rounds: u8,
    /// Zero disables the individual turn clock.
    pub guess_seconds: u16,
}

impl Default for TurnBasedRules {
    fn default() -> Self {
        Self {
            options: GuessOptions::default(),
            rounds: 1,
            guess_seconds: 60,
        }
    }
}

impl TurnBasedRules {
    pub const GUESS_SECONDS_OPTIONS: [u16; 5] = [15, 30, 60, 120, 0];

    pub(super) fn is_valid(self) -> bool {
        self.options.is_valid()
            && (1..=5).contains(&self.rounds)
            && Self::GUESS_SECONDS_OPTIONS.contains(&self.guess_seconds)
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct RaceRules {
    pub options: GuessOptions,
    pub games: u8,
    pub seconds: u16,
}

impl Default for RaceRules {
    fn default() -> Self {
        Self {
            options: GuessOptions::default(),
            games: 5,
            seconds: 60,
        }
    }
}

impl RaceRules {
    pub const SECONDS_OPTIONS: [u16; 5] = [40, 60, 90, 120, 180];
    pub const GAMES_OPTIONS: [u8; 3] = [3, 5, 10];

    pub(super) fn is_valid(self) -> bool {
        self.options.is_valid()
            && Self::GAMES_OPTIONS.contains(&self.games)
            && Self::SECONDS_OPTIONS.contains(&self.seconds)
    }
}
