//! 客户端布局、动画节奏、资源路径与配色常量。

use bevy::prelude::*;

pub const DESIGN_WIDTH: f32 = 1280.0;
pub const DESIGN_HEIGHT: f32 = 720.0;
pub const HAND_CARD_REVEAL: f32 = 28.0;
pub const TABLE_CARD_REVEAL: f32 = 27.0;
pub const TABLE_SCORE_CARD_REVEAL: f32 = 8.0;
pub const SCORE_ROLL_DELAY: f32 = 0.42;
pub const SCORE_ROLL_DURATION: f32 = 0.72;
pub const MIN_TABLE_BRIGHTNESS: f32 = 0.1;
pub const MAX_TABLE_BRIGHTNESS: f32 = 1.25;
pub const MIN_TABLE_VIGNETTE: f32 = 0.0;
pub const MAX_TABLE_VIGNETTE: f32 = 0.75;
pub const DEFAULT_TABLE_VIGNETTE: f32 = 0.38;
pub const TABLE_BG: Color = Color::srgb(0.025, 0.105, 0.075);
pub const HEADER_BG: Color = Color::srgb(0.025, 0.075, 0.06);
pub const PANEL: Color = Color::srgba(0.055, 0.19, 0.135, 0.96);
pub const PANEL_ALT: Color = Color::srgba(0.075, 0.24, 0.17, 0.96);
pub const TEXT: Color = Color::srgb(0.94, 0.97, 0.95);
pub const MUTED: Color = Color::srgb(0.63, 0.73, 0.68);
pub const ACCENT: Color = Color::srgb(0.96, 0.72, 0.20);
pub const READY: Color = Color::srgb(0.34, 0.86, 0.53);
pub const DANGER: Color = Color::srgb(0.96, 0.39, 0.34);
pub const BORDER: Color = Color::srgba(0.72, 0.88, 0.79, 0.18);
