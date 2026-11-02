use super::super::*;
use super::panel::ChatAuxiliaryAction;
use crate::app::presentation::{ACCENT, MUTED, TEXT};
use crate::app::runtime::UiAssets;
use crate::app::shell::{ChatUiAction, SocialUiAction, UiAction};
use bevy::prelude::*;
use bevy::ui_widgets::Button;
use leocard_ui::{ButtonTint, TextureButton, UiComposer};

pub(super) fn side_control_node(top: f32) -> Node {
    Node {
        position_type: PositionType::Absolute,
        left: px(-32),
        top: px(top),
        width: px(32),
        min_width: px(32),
        max_width: px(32),
        height: px(32),
        min_height: px(32),
        max_height: px(32),
        flex_shrink: 0.0,
        align_items: AlignItems::Center,
        justify_content: JustifyContent::Center,
        ..default()
    }
}

pub(super) fn add_chat_toggle(
    commands: &mut UiComposer,
    panel: Entity,
    chat: &ChatPanelState,
    assets: &UiAssets,
) {
    commands.child(
        panel,
        (
            Button,
            UiAction::Chat(ChatUiAction::TogglePanel),
            side_control_node(154.0),
            assets.tabletop.fold_image(chat.open),
            ChatToggleIcon,
            side_button_tint(),
        ),
    );
}

pub(super) fn add_auto_play_toggle(
    commands: &mut UiComposer,
    panel: Entity,
    enabled: bool,
    assets: &UiAssets,
) {
    let image = assets.tabletop.robot_image();
    TextureButton {
        node: side_control_node(194.0),
        normal: image
            .clone()
            .with_color(if enabled { ACCENT } else { Color::WHITE }),
        hovered: image.with_color(if enabled {
            Color::srgb(1.25, 0.95, 0.40)
        } else {
            Color::srgb(1.25, 1.25, 1.25)
        }),
        action: Some(UiAction::Social(SocialUiAction::ToggleAutoPlay)),
        label: None,
        font_size: 13.0,
        color: TEXT,
        icon: None,
    }
    .mount(commands, panel, &assets.font);
}

pub(super) fn side_button_tint() -> ButtonTint {
    ButtonTint {
        normal: Color::WHITE,
        hovered: Color::srgb(1.25, 1.25, 1.25),
        pressed: Color::srgb(0.82, 0.82, 0.82),
    }
}

pub(super) fn add_auxiliary_actions(
    commands: &mut UiComposer,
    panel: Entity,
    actions: &[ChatAuxiliaryAction],
    assets: &UiAssets,
) {
    for (index, action) in actions.iter().enumerate() {
        add_auxiliary_action(commands, panel, assets, action, index);
    }
}

fn add_auxiliary_action(
    commands: &mut UiComposer,
    panel: Entity,
    assets: &UiAssets,
    auxiliary: &ChatAuxiliaryAction,
    index: usize,
) {
    let enabled = auxiliary.action.is_some();
    let highlighted = auxiliary.highlighted && enabled;
    let tint = if highlighted { ACCENT } else { Color::WHITE };
    let image = assets.tabletop.side_button_image();
    let (_, label) = TextureButton {
        node: side_control_node(234.0 + index as f32 * 40.0),
        normal: image
            .clone()
            .with_color(tint.with_alpha(if enabled { 1.0 } else { 0.55 })),
        hovered: image.with_color(if highlighted {
            Color::srgb(1.25, 0.95, 0.40)
        } else {
            Color::srgb(1.25, 1.25, 1.25)
        }),
        action: auxiliary.action.clone(),
        label: Some(auxiliary.label),
        font_size: 16.0,
        color: if enabled { TEXT } else { MUTED },
        icon: None,
    }
    .mount(commands, panel, &assets.title_font);
    commands.entity(label).insert((
        TextLayout::no_wrap(),
        TextShadow {
            offset: Vec2::new(0.7, 1.0),
            color: Color::BLACK.with_alpha(0.85),
        },
    ));
}
