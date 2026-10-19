use crate::GuessMode;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum MatchLength {
    /// Every participant takes the opening turn once per round.
    Rounds(u8),
    /// A fixed question count shared by every participant.
    Games(u8),
    Graph,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct MatchProgress {
    pub game_index: u8,
    pub player_count: u8,
    pub length: MatchLength,
}

impl MatchProgress {
    pub fn mode(self) -> GuessMode {
        match self.length {
            MatchLength::Rounds(_) => GuessMode::TurnBased,
            MatchLength::Games(_) => GuessMode::Race,
            MatchLength::Graph => GuessMode::Graph,
        }
    }

    pub fn round_count(self) -> Option<u8> {
        match self.length {
            MatchLength::Rounds(rounds) => Some(rounds),
            MatchLength::Games(_) | MatchLength::Graph => None,
        }
    }

    pub fn current_round(self) -> u8 {
        match self.length {
            MatchLength::Rounds(_) => self.game_index / self.player_count + 1,
            MatchLength::Games(_) | MatchLength::Graph => 1,
        }
    }

    pub fn game_in_round(self) -> u8 {
        match self.length {
            MatchLength::Rounds(_) => self.game_index % self.player_count + 1,
            MatchLength::Games(_) | MatchLength::Graph => self.game_index + 1,
        }
    }

    pub fn total_games(self) -> u8 {
        match self.length {
            MatchLength::Rounds(rounds) => self.player_count * rounds,
            MatchLength::Games(games) => games,
            MatchLength::Graph => 1,
        }
    }

    pub fn is_last_game(self) -> bool {
        self.game_index + 1 == self.total_games()
    }
}
