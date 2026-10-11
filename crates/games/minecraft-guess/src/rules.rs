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
}

impl Default for MinecraftGuessRuleSet {
    fn default() -> Self {
        Self {
            pool: GuessPool::default(),
            candidate_hints: true,
            allow_unknown_guesses: false,
            rounds: 1,
        }
    }
}

impl MinecraftGuessRuleSet {
    pub fn is_valid(self) -> bool {
        (1..=5).contains(&self.rounds) && !(self.candidate_hints && self.allow_unknown_guesses)
    }
}
