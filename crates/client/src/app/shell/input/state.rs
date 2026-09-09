//! 通用文本输入与手牌拖选的运行时状态。
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

#[derive(Resource, Default)]
pub(crate) struct DeveloperHandInput {
    pub value: String,
    pub focused: bool,
}

#[derive(Resource, Default)]
pub(crate) struct CardDragSelection {
    pub active: bool,
    pub anchor: usize,
    pub current: usize,
    pub select: bool,
}

impl CardDragSelection {
    pub(crate) fn contains(&self, index: usize) -> bool {
        self.active
            && (self.anchor.min(self.current)..=self.anchor.max(self.current)).contains(&index)
    }
}

#[derive(Component)]
pub(crate) struct HandCardSelectionOverlay {
    pub index: usize,
}

#[derive(Component)]
pub(crate) struct DeveloperHandInputText {
    pub placeholder: &'static str,
}

#[derive(Component)]
pub(crate) struct DeveloperHandInputField;
