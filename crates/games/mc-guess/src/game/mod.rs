mod guess;
mod match_state;
mod scoring;
mod single;
mod state;
#[cfg(test)]
mod tests;
mod types;

pub use guess::{McGuessRecord, McGuessResult};
pub use match_state::{McGuessMatchLength, McGuessMatchProgress, McGuessMatchState};
pub use scoring::{McGuessScoreGain, reference_point_deltas};
pub use state::McGuessGameState;
pub use types::{McGuessGameError, McGuessRoundView};
