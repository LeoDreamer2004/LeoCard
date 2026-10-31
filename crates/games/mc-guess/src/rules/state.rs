use super::{McGuessGraphRules, McGuessOptions, McGuessRaceRules, McGuessTurnBasedRules};
use serde::{Deserialize, Serialize};

pub const MAX_GUESS_CHARACTERS: usize = 16;

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub enum McGuessMode {
    #[default]
    TurnBased,
    Race,
    Graph,
}

impl McGuessMode {
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
pub struct McGuessRuleSet {
    pub mode: McGuessMode,
    pub turn_based: McGuessTurnBasedRules,
    pub race: McGuessRaceRules,
    pub graph: McGuessGraphRules,
}

impl McGuessRuleSet {
    pub const MIN_PLAYERS: u8 = 2;
    pub const MAX_PLAYERS: u8 = 8;

    pub fn options(&self) -> &McGuessOptions {
        match self.mode {
            McGuessMode::TurnBased => &self.turn_based.options,
            McGuessMode::Race => &self.race.options,
            McGuessMode::Graph => &self.graph.options,
        }
    }

    pub fn options_mut(&mut self) -> &mut McGuessOptions {
        match self.mode {
            McGuessMode::TurnBased => &mut self.turn_based.options,
            McGuessMode::Race => &mut self.race.options,
            McGuessMode::Graph => &mut self.graph.options,
        }
    }

    pub fn guess_seconds(self) -> u16 {
        match self.mode {
            McGuessMode::TurnBased => self.turn_based.guess_seconds,
            McGuessMode::Race => self.race.seconds,
            McGuessMode::Graph => self.graph.guess_seconds,
        }
    }

    pub fn is_valid(self) -> bool {
        self.turn_based.is_valid() && self.race.is_valid() && self.graph.is_valid()
    }
}
