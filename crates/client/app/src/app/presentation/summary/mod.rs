//! 跨游戏共用的终局结算表现。

mod layout;
mod ranked;
mod state;
mod systems;
#[cfg(test)]
mod tests;

pub(crate) use layout::spawn_centered_summary;
pub(crate) use ranked::{RankedSummary, SummaryPlayer, SummaryRows, add_ranked_summary};
pub(crate) use state::*;
pub(crate) use systems::*;
