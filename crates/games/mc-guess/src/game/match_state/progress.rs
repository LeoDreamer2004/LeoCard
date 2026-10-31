use crate::McGuessMode;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum McGuessMatchLength {
    /// Every participant takes the opening turn once per round.
    Rounds(u8),
    /// A fixed question count shared by every participant.
    Games(u8),
    /// A fixed graph count, with a different opening player for each graph.
    Graph(u8),
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct McGuessMatchProgress {
    pub game_index: u8,
    pub player_count: u8,
    pub length: McGuessMatchLength,
}

impl McGuessMatchProgress {
    pub fn mode(self) -> McGuessMode {
        match self.length {
            McGuessMatchLength::Rounds(_) => McGuessMode::TurnBased,
            McGuessMatchLength::Games(_) => McGuessMode::Race,
            McGuessMatchLength::Graph(_) => McGuessMode::Graph,
        }
    }

    pub fn round_count(self) -> Option<u8> {
        match self.length {
            McGuessMatchLength::Rounds(rounds) => Some(rounds),
            McGuessMatchLength::Games(_) | McGuessMatchLength::Graph(_) => None,
        }
    }

    pub fn current_round(self) -> u8 {
        match self.length {
            McGuessMatchLength::Rounds(_) => self.game_index / self.player_count + 1,
            McGuessMatchLength::Games(_) | McGuessMatchLength::Graph(_) => 1,
        }
    }

    pub fn game_in_round(self) -> u8 {
        match self.length {
            McGuessMatchLength::Rounds(_) => self.game_index % self.player_count + 1,
            McGuessMatchLength::Games(_) | McGuessMatchLength::Graph(_) => self.game_index + 1,
        }
    }

    pub fn total_games(self) -> u8 {
        match self.length {
            McGuessMatchLength::Rounds(rounds) => self.player_count * rounds,
            McGuessMatchLength::Games(games) => games,
            McGuessMatchLength::Graph(games) => games,
        }
    }

    pub fn is_last_game(self) -> bool {
        self.game_index + 1 == self.total_games()
    }
}
