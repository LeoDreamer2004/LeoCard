use super::{DeveloperHandInput, DeveloperHandInputField};
use crate::app::presentation::TextInput;
use crate::app::runtime::UiAssets;
use bevy::prelude::*;

pub(crate) fn add_developer_hand_input(
    commands: &mut Commands,
    parent: Entity,
    input: &DeveloperHandInput,
    placeholder: &'static str,
    position: Vec2,
    assets: &UiAssets,
) {
    let mut field = TextInput::new("developer.hand", &input.value);
    field.placeholder = placeholder;
    field.max_characters = 192;
    field.filter = |character| character.is_ascii_alphanumeric();
    field.font_size = 14.0;
    let editor = field.spawn(
        commands,
        parent,
        Node {
            position_type: PositionType::Absolute,
            left: px(position.x),
            bottom: px(position.y),
            width: px(300),
            height: px(48),
            padding: UiRect::axes(px(12), px(7)),
            ..default()
        },
        assets,
    );
    commands.entity(editor).insert(DeveloperHandInputField);
}
