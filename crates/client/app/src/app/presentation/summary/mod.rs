//! 跨游戏共用的终局结算表现。

mod ranked;
mod state;
mod systems;
#[cfg(test)]
mod tests;

pub(crate) use ranked::{RankedSummary, SummaryPlayer, add_ranked_summary};
pub(crate) use state::*;
pub(crate) use systems::*;
