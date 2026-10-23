use super::counts::CardCounts;
use crate::app::presentation::CardCounterWindowState;

#[derive(Default)]
pub(crate) struct QiGuiCounterUi {
    pub window: CardCounterWindowState,
    pub(super) counts: CardCounts,
}
