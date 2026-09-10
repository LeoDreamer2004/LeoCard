use crate::app::presentation::{ACCENT, BORDER, HEADER_BG, MUTED, TEXT, add_text};
use crate::app::runtime::UiAssets;
use crate::app::shell::{
    DeveloperHandInput, DeveloperHandInputField, DeveloperHandInputText, DeveloperUiAction,
    UiAction, developer_hand_input_label,
};
use bevy::prelude::*;

pub(crate) fn add_developer_hand_input(
    commands: &mut Commands,
    parent: Entity,
    input: &DeveloperHandInput,
    placeholder: &'static str,
    position: Vec2,
    assets: &UiAssets,
) {
    let field = commands
        .spawn((
            Button,
            UiAction::Developer(DeveloperUiAction::FocusHandInput),
            Node {
                position_type: PositionType::Absolute,
                left: px(position.x),
                bottom: px(position.y),
                width: px(300),
                height: px(48),
                padding: UiRect::axes(px(12), px(7)),
                align_items: AlignItems::Center,
                border: UiRect::all(px(if input.focused { 2 } else { 1 })),
                border_radius: BorderRadius::all(px(7)),
                ..default()
            },
            BackgroundColor(HEADER_BG.with_alpha(0.94)),
            BorderColor::all(if input.focused { ACCENT } else { BORDER }),
            DeveloperHandInputField,
        ))
        .id();
    commands.entity(parent).add_child(field);
    let text = add_text(
        commands,
        field,
        developer_hand_input_label(input, placeholder),
        14.0,
        if input.value.is_empty() { MUTED } else { TEXT },
        assets,
    );
    commands
        .entity(text)
        .insert(DeveloperHandInputText { placeholder });
}
