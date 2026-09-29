use crate::app::presentation::{MUTED, TEXT, add_text};
use crate::app::runtime::UiAssets;
use crate::app::shell::{
    DeveloperHandInput, DeveloperHandInputField, DeveloperHandInputText, DeveloperUiAction,
    UiAction, developer_hand_input_label,
};
use bevy::prelude::*;
use bevy::ui::VisualBox;

pub(crate) fn add_developer_hand_input(
    commands: &mut Commands,
    parent: Entity,
    input: &DeveloperHandInput,
    placeholder: &'static str,
    position: Vec2,
    assets: &UiAssets,
) {
    let mut image = ImageNode::new(if input.focused {
        assets.home.focused_input.clone()
    } else {
        assets.home.input.clone()
    })
    .with_mode(NodeImageMode::Sliced(TextureSlicer {
        border: BorderRect::all(32.0),
        center_scale_mode: SliceScaleMode::Stretch,
        sides_scale_mode: SliceScaleMode::Stretch,
        max_corner_scale: 0.45,
    }));
    image.visual_box = VisualBox::BorderBox;
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
                ..default()
            },
            image,
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
    commands.entity(text).insert((
        DeveloperHandInputText { placeholder },
        TextBackgroundColor(if input.selected_all {
            Color::srgb(0.20, 0.42, 0.72)
        } else {
            Color::NONE
        }),
    ));
}
