mod automation;
mod commands;
mod lifecycle;
mod snapshot;
mod state;
mod statistics;
mod support;
#[cfg(test)]
mod tests;

pub use state::*;
use support::*;

mod win_feedback;
use win_feedback::MahjongWinFeedback;
