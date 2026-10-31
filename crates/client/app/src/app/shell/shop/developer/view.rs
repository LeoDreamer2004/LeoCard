use super::state::CoinInputField;
use crate::app::runtime::UiAssets;
use bevy::prelude::*;
use bevy::ui_widgets::SelectAllOnFocus;
use leocard_ui::{TextInput, UiComposer};

pub(crate) fn add_coin_input(
    commands: &mut UiComposer,
    parent: Entity,
    coins: u32,
    assets: &UiAssets,
) {
    let value = coins.to_string();
    let mut input = TextInput::new("developer.coins", &value);
    input.max_characters = 10;
    input.filter = |character| character.is_ascii_digit();
    input.font_size = 16.0;
    let editor = input.spawn(
        commands,
        parent,
        Node {
            width: px(160),
            height: px(40),
            padding: UiRect::axes(px(12), px(6)),
            ..default()
        },
        assets.text_input_style(),
    );
    commands
        .entity(editor)
        .insert((CoinInputField, SelectAllOnFocus));
}
