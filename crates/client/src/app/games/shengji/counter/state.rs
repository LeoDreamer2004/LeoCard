use super::counts::CardCounts;
use bevy::prelude::*;

pub(crate) struct ShengjiCounterUi {
    pub drawer_open: bool,
    pub enabled: bool,
    pub active: bool,
    pub position: Vec2,
    pub(super) counts: CardCounts,
    pub(super) grab: Option<Vec2>,
}

impl Default for ShengjiCounterUi {
    fn default() -> Self {
        Self {
            drawer_open: false,
            enabled: false,
            active: false,
            position: Vec2::new(34.0, 296.0),
            counts: CardCounts::default(),
            grab: None,
        }
    }
}

#[derive(Component)]
pub(in super::super) struct CounterWindow;

#[derive(Component)]
pub(in super::super) struct CounterDragHandle;
