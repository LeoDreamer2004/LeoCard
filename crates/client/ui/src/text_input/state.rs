use bevy::prelude::*;

#[derive(Component, Clone, Copy, Eq, Hash, PartialEq)]
pub(super) struct TextInputKey(pub &'static str);

#[derive(Component)]
pub(super) struct TextInputSlot {
    pub editor: Entity,
}

#[derive(Component)]
pub(super) struct InputPlaceholder;

#[derive(Clone, Copy, Eq, PartialEq)]
pub enum TextInputAction {
    Submit,
    Cancel,
}

#[derive(Message)]
pub struct TextInputEvent {
    pub entity: Entity,
    pub action: TextInputAction,
}

#[derive(SystemSet, Debug, Clone, Eq, Hash, PartialEq)]
pub enum TextInputSet {
    Publish,
}

#[cfg(target_os = "android")]
#[derive(Component)]
pub struct AndroidEditor {
    pub title: String,
    pub numeric: bool,
    pub filter: fn(char) -> bool,
}
