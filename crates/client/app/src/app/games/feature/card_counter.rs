use crate::app::presentation::{CardCounterCell, CardCounterWindowState};
use crate::app::runtime::UiAssets;

pub(crate) type CounterRows = [[Option<CardCounterCell>; 14]; 4];

/// Counts use only the recipient's visible cards and retain game-specific ordering/colors.
pub(crate) trait CardCounter {
    type Context;

    fn counter_window(&self) -> &CardCounterWindowState;

    fn counter_rows(&self, context: &Self::Context, assets: &UiAssets) -> CounterRows;
}
