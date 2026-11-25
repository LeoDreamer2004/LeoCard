use bevy::prelude::*;

#[derive(Component)]
pub struct SwitchState {
    pub(super) selected: bool,
    pub(super) enabled: bool,
    pub(super) scale: f32,
    pub(super) off_color: Color,
    pub(super) on_color: Color,
    pub(super) mask: Entity,
    pub(super) mask_color: Color,
}

/// Retain the thumb's progress while its owner is reused by the scene composer.
#[derive(Clone, Component)]
pub struct SwitchAnimation {
    pub(super) owner: Entity,
    pub(super) progress: f32,
}

impl Default for SwitchAnimation {
    fn default() -> Self {
        Self {
            owner: Entity::PLACEHOLDER,
            progress: 0.0,
        }
    }
}

impl SwitchAnimation {
    pub fn same_owner(&self, other: &Self) -> bool {
        self.owner == other.owner
    }
}
