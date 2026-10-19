use super::{GraphRules, GuessOptions, RaceRules, TurnBasedRules};
use serde::{Deserialize, Serialize};

pub const MAX_GUESS_CHARACTERS: usize = 16;

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub enum GuessMode {
    #[default]
    TurnBased,
    Race,
    Graph,
}

impl GuessMode {
    pub fn label(self) -> &'static str {
        match self {
            Self::TurnBased => "轮流竞猜",
            Self::Race => "抢答竞速",
            Self::Graph => "关系谜图",
        }
    }
}

/// Mode configurations travel together so switching modes preserves every option.
#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct MinecraftGuessRuleSet {
    pub mode: GuessMode,
    pub turn_based: TurnBasedRules,
    pub race: RaceRules,
    pub graph: GraphRules,
}

impl MinecraftGuessRuleSet {
    pub const MIN_PLAYERS: u8 = 2;
    pub const MAX_PLAYERS: u8 = 8;

    pub fn options(&self) -> &GuessOptions {
        match self.mode {
            GuessMode::TurnBased => &self.turn_based.options,
            GuessMode::Race => &self.race.options,
            GuessMode::Graph => &self.graph.options,
        }
    }

    pub fn options_mut(&mut self) -> &mut GuessOptions {
        match self.mode {
            GuessMode::TurnBased => &mut self.turn_based.options,
            GuessMode::Race => &mut self.race.options,
            GuessMode::Graph => &mut self.graph.options,
        }
    }

    pub fn guess_seconds(self) -> u16 {
        match self.mode {
            GuessMode::TurnBased => self.turn_based.guess_seconds,
            GuessMode::Race => self.race.seconds,
            GuessMode::Graph => 0,
        }
    }

    pub fn is_valid(self) -> bool {
        self.turn_based.is_valid() && self.race.is_valid() && self.graph.is_valid()
    }
}
