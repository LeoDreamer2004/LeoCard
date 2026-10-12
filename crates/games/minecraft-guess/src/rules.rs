use serde::{Deserialize, Serialize};

pub const MAX_GUESS_CHARACTERS: usize = 10;

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub enum GuessPool {
    Blocks,
    Items,
    #[default]
    Mixed,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct MinecraftGuessRuleSet {
    pub pool: GuessPool,
    pub candidate_hints: bool,
    pub allow_unknown_guesses: bool,
    pub rounds: u8,
    /// Seconds allowed for each guess; zero disables the turn clock.
    pub guess_seconds: u16,
}

impl Default for MinecraftGuessRuleSet {
    fn default() -> Self {
        Self {
            pool: GuessPool::default(),
            candidate_hints: true,
            allow_unknown_guesses: false,
            rounds: 1,
            guess_seconds: 60,
        }
    }
}

impl MinecraftGuessRuleSet {
    pub const MIN_PLAYERS: u8 = 2;
    pub const MAX_PLAYERS: u8 = 8;

    pub const GUESS_SECONDS_OPTIONS: [u16; 5] = [0, 15, 30, 60, 120];

    pub fn is_valid(self) -> bool {
        (1..=5).contains(&self.rounds)
            && !(self.candidate_hints && self.allow_unknown_guesses)
            && Self::GUESS_SECONDS_OPTIONS.contains(&self.guess_seconds)
    }
}
