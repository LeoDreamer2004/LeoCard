//! 客户端布局、动画节奏、资源路径与配色常量。

use bevy::prelude::*;

pub(crate) const DESIGN_WIDTH: f32 = 1280.0;
pub(crate) const DESIGN_HEIGHT: f32 = 720.0;
pub(crate) const HAND_CARD_REVEAL: f32 = 28.0;
pub(crate) const TABLE_CARD_REVEAL: f32 = 27.0;
pub(crate) const TABLE_SCORE_CARD_REVEAL: f32 = 8.0;
pub(crate) const SCORE_ROLL_DELAY: f32 = 0.42;
pub(crate) const SCORE_ROLL_DURATION: f32 = 0.72;
pub(crate) const MIN_TABLE_BRIGHTNESS: f32 = 0.1;
pub(crate) const MAX_TABLE_BRIGHTNESS: f32 = 1.25;
pub(crate) const MIN_TABLE_VIGNETTE: f32 = 0.0;
pub(crate) const MAX_TABLE_VIGNETTE: f32 = 0.75;
pub(crate) const DEFAULT_TABLE_VIGNETTE: f32 = 0.38;
pub(crate) const TABLE_BG: Color = Color::srgb(0.025, 0.105, 0.075);
pub(crate) const HEADER_BG: Color = Color::srgb(0.025, 0.075, 0.06);
pub(crate) const PANEL: Color = Color::srgba(0.055, 0.19, 0.135, 0.96);
pub(crate) const PANEL_ALT: Color = Color::srgba(0.075, 0.24, 0.17, 0.96);
pub(crate) const TEXT: Color = Color::srgb(0.94, 0.97, 0.95);
pub(crate) const MUTED: Color = Color::srgb(0.63, 0.73, 0.68);
pub(crate) const ACCENT: Color = Color::srgb(0.96, 0.72, 0.20);
pub(crate) const READY: Color = Color::srgb(0.34, 0.86, 0.53);
pub(crate) const DANGER: Color = Color::srgb(0.96, 0.39, 0.34);
pub(crate) const BORDER: Color = Color::srgba(0.72, 0.88, 0.79, 0.18);
