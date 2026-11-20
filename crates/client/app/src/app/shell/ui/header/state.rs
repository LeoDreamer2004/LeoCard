use crate::app::presentation::game_side_control_scale;
use bevy::prelude::*;
use leocard_platform::is_mobile;

pub(super) fn height() -> f32 {
    66.0 * game_side_control_scale()
}

pub(super) fn avatar_size() -> f32 {
    38.0 * game_side_control_scale()
}

pub(super) fn right_inset() -> f32 {
    if is_mobile() { 36.0 } else { 12.0 }
}

pub(super) fn item_width() -> f32 {
    44.0 * game_side_control_scale()
}

pub(super) fn item_gap() -> f32 {
    8.0 * game_side_control_scale()
}

#[derive(Component, Default)]
pub(super) struct GameHeaderMotion {
    pub expanded: bool,
    pub progress: f32,
}

#[derive(Component)]
pub(super) struct GameHeaderAvatar;

#[derive(Component)]
pub(super) struct GameHeaderItem {
    /// Items unfold from the avatar, starting with the nearest one.
    pub slot: usize,
    pub applied: Option<f32>,
}

impl GameHeaderItem {
    pub fn right(&self) -> f32 {
        right_inset() + avatar_size() + item_gap() + self.slot as f32 * (item_width() + item_gap())
    }

    pub fn folded_offset(&self) -> f32 {
        self.right() + item_width() * 0.5 - (right_inset() + avatar_size() * 0.5)
    }
}
