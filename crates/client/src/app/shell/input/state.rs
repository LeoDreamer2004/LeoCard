//! 通用文本输入与手牌拖选的运行时状态。

use super::*;

#[derive(Resource)]
pub struct UiZoom {
    pub manual: f32,
}

impl Default for UiZoom {
    fn default() -> Self {
        Self { manual: 1.0 }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum InputField {
    PlayerName,
    HostPort,
    JoinAddress,
}

#[derive(Resource, Default)]
pub struct DeveloperHandInput {
    pub value: String,
    pub focused: bool,
}

#[derive(Resource, Default)]
pub struct CardDragSelection {
    pub active: bool,
    pub anchor: usize,
    pub current: usize,
    pub select: bool,
}

impl CardDragSelection {
    pub fn contains(&self, index: usize) -> bool {
        self.active
            && (self.anchor.min(self.current)..=self.anchor.max(self.current)).contains(&index)
    }
}

#[derive(Component)]
pub struct HandCardSelectionOverlay {
    pub index: usize,
}

#[derive(Component)]
pub struct DeveloperHandInputText {
    pub placeholder: &'static str,
}

#[derive(Component)]
pub struct DeveloperHandInputField;
