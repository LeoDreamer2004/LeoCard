//! Android 使用无牌山的大牌布局，桌面端保留原来的尺寸和坐标。

use bevy::prelude::*;

pub(super) struct MahjongLayout;

impl MahjongLayout {
    pub const BUTTON_WIDTH: f32 = if cfg!(target_os = "android") {
        212.0
    } else {
        164.0
    };
    pub const BUTTON_HEIGHT: f32 = if cfg!(target_os = "android") {
        74.0
    } else {
        54.0
    };
    pub const BUTTON_FONT: f32 = if cfg!(target_os = "android") {
        26.0
    } else {
        20.0
    };

    pub const OWN_WIDTH: f32 = if cfg!(target_os = "android") {
        78.0
    } else {
        56.0
    };
    pub const OWN_HEIGHT: f32 = if cfg!(target_os = "android") {
        106.0
    } else {
        76.0
    };
    pub const OWN_STEP: f32 = if cfg!(target_os = "android") {
        70.0
    } else {
        50.0
    };
    pub const OWN_ROW_HEIGHT: f32 = if cfg!(target_os = "android") {
        118.0
    } else {
        88.0
    };
    pub const OWN_ROW_WIDTH: f32 = if cfg!(target_os = "android") {
        1060.0
    } else {
        760.0
    };
    pub const DRAW_GAP: f32 = if cfg!(target_os = "android") {
        30.0
    } else {
        22.0
    };
    pub const MINI_WIDTH: f32 = if cfg!(target_os = "android") {
        36.0
    } else {
        27.0
    };
    pub const MINI_HEIGHT: f32 = if cfg!(target_os = "android") {
        50.0
    } else {
        37.0
    };
    pub const MINI_STEP: f32 = if cfg!(target_os = "android") {
        32.0
    } else {
        24.0
    };
    pub const REMOTE_HEIGHT: f32 = if cfg!(target_os = "android") {
        72.0
    } else {
        54.0
    };
    pub const RIVER_WIDTH: f32 = if cfg!(target_os = "android") {
        44.0
    } else {
        33.0
    };
    pub const RIVER_HEIGHT: f32 = if cfg!(target_os = "android") {
        60.0
    } else {
        45.0
    };
    pub const RIVER_STEP: f32 = if cfg!(target_os = "android") {
        40.0
    } else {
        30.0
    };
    pub const RIVER_COLUMNS: usize = if cfg!(target_os = "android") { 12 } else { 6 };
    pub const RIVER_ROWS_HEIGHT: f32 = if cfg!(target_os = "android") {
        112.0
    } else {
        126.0
    };
    pub const RIVER_ROWS_WIDTH: f32 = if cfg!(target_os = "android") {
        484.0
    } else {
        182.0
    };

    pub fn river_origin(relative: u8) -> Vec2 {
        let (x, y) = if cfg!(target_os = "android") {
            match relative {
                0 => (398.0, 350.0),
                1 => (768.0, 264.0),
                2 => (398.0, 110.0),
                _ => (28.0, 264.0),
            }
        } else {
            match relative {
                0 => (549.0, 382.0),
                1 => (735.0, 245.0),
                2 => (533.0, 150.0),
                _ => (331.0, 245.0),
            }
        };
        Vec2::new(x, y)
    }
}
