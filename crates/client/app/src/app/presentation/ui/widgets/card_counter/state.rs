use bevy::prelude::*;
use leocard_ui::DraggableWindowState;

pub(crate) struct CardCounterWindowState {
    pub drawer_open: bool,
    pub enabled: bool,
    pub active: bool,
    pub drag: DraggableWindowState,
}

impl Default for CardCounterWindowState {
    fn default() -> Self {
        Self {
            drawer_open: false,
            enabled: false,
            active: false,
            drag: DraggableWindowState {
                position: Vec2::new(34.0, 296.0),
                ..Default::default()
            },
        }
    }
}
