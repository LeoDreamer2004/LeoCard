use bevy::prelude::*;

pub(super) const HEIGHT: f32 = 66.0;
pub(super) const AVATAR_SIZE: f32 = 38.0;
pub(super) const RIGHT_INSET: f32 = 12.0;
pub(super) const ITEM_WIDTH: f32 = 44.0;
pub(super) const ITEM_GAP: f32 = 8.0;

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
        RIGHT_INSET + AVATAR_SIZE + ITEM_GAP + self.slot as f32 * (ITEM_WIDTH + ITEM_GAP)
    }

    pub fn folded_offset(&self) -> f32 {
        self.right() + ITEM_WIDTH * 0.5 - (RIGHT_INSET + AVATAR_SIZE * 0.5)
    }
}
