mod automation;
mod commands;
mod lifecycle;
mod snapshot;
mod state;
mod support;
#[cfg(test)]
mod tests;

use state::MAHJONG_DEAL_INTERVAL;
pub use state::MahjongSession;
use support::*;
