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

#[derive(Component)]
pub struct PlatformEditor {
    pub title: String,
    pub password: bool,
    pub max_characters: usize,
    pub numeric: bool,
    pub multiline: bool,
    pub filter: fn(char) -> bool,
}
