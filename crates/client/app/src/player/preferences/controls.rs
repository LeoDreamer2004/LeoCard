use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Default, Deserialize, Serialize)]
#[serde(default)]
pub struct ControlPreferences {
    pub mahjong_right_click_draw_discard: bool,
}
