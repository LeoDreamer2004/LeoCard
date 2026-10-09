use bevy::prelude::*;
use std::collections::HashMap;

#[derive(Component, Clone, Copy, Eq, Hash, PartialEq)]
pub(super) struct TextInputKey(pub &'static str);

#[derive(Component)]
pub(super) struct TextInputSlot {
    pub key: TextInputKey,
    pub editor: Entity,
}

#[derive(Component)]
pub(super) struct InputPlaceholder;

#[derive(Resource, Default)]
pub(super) struct RetainedEditors {
    pub editors: HashMap<TextInputKey, Entity>,
    pub rebuilding: bool,
}

#[derive(Clone, Copy, Eq, PartialEq)]
pub(crate) enum TextInputAction {
    Submit,
    Cancel,
}

#[derive(Message)]
pub(crate) struct TextInputEvent {
    pub entity: Entity,
    pub action: TextInputAction,
}

#[derive(SystemSet, Debug, Clone, Eq, Hash, PartialEq)]
pub(crate) enum TextInputSet {
    Publish,
}
