//! 通用输入状态。
use bevy::prelude::*;

#[derive(Resource)]
pub(crate) struct UiZoom {
    pub manual: f32,
}

impl Default for UiZoom {
    fn default() -> Self {
        Self { manual: 1.0 }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum InputField {
    PlayerName,
    HostPort,
    JoinAddress,
}
