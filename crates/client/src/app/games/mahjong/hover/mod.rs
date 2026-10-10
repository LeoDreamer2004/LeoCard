mod scores;
mod systems;
mod view;
mod waits;

use systems::MahjongWaitPopupLink;
pub(super) use systems::{MahjongHoverHandKind, MahjongMatchingTileKind, sync_mahjong_hover_hints};
pub(super) use systems::{MahjongReadyHint, MahjongReadyHintState};
pub(super) use view::{add_mahjong_ready_hint, add_mahjong_wait_popup};
use waits::mahjong_current_waits;
pub(super) use waits::mahjong_discard_waits;

pub(crate) use scores::MahjongFanCalculator;
pub(super) use scores::sync_fan_calculator;
