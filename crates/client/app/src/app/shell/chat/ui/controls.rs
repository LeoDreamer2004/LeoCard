use super::super::*;
use super::panel::{ChatAuxiliaryAction, cozy_chat_button_image};
use crate::app::presentation::{MUTED, TEXT};
use crate::app::runtime::UiAssets;
use crate::app::shell::{ChatUiAction, SocialUiAction, UiAction};
use bevy::prelude::*;
use bevy::ui_widgets::Button;
use leocard_ui::{TextureButton, UiComposer};

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
            Node {
                position_type: PositionType::Absolute,
                left: px(-32),
                top: px(147),
                width: px(32),
                min_width: px(32),
                max_width: px(32),
                height: px(46),
                min_height: px(46),
                max_height: px(46),
                flex_shrink: 0.0,
                ..default()
            },
            ImageNode::new(if chat.open {
                assets.home.rule_right.clone()
            } else {
                assets.home.rule_left.clone()
            }),
            ChatToggleIcon,
        ),
    );
}

pub(super) fn add_auto_play_toggle(
    commands: &mut UiComposer,
    panel: Entity,
    enabled: bool,
    assets: &UiAssets,
) {
    TextureButton {
        node: Node {
            position_type: PositionType::Absolute,
            left: px(-32),
            top: px(194),
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
        },
        normal: cozy_chat_button_image(if enabled {
            assets.home.purple_button_compact.clone()
        } else {
            assets.home.button.clone()
        }),
        hovered: cozy_chat_button_image(assets.home.purple_button_compact.clone()),
        action: Some(UiAction::Social(SocialUiAction::ToggleAutoPlay)),
        label: None,
        font_size: 13.0,
        color: TEXT,
        icon: Some((assets.controls.robot_icon.clone(), 25.0)),
    }
    .mount(commands, panel, &assets.font);
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
    TextureButton {
        node: Node {
            position_type: PositionType::Absolute,
            left: px(-32),
            top: px(234.0 + index as f32 * 40.0),
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
        },
        normal: cozy_chat_button_image(if auxiliary.highlighted && enabled {
            assets.home.purple_button_compact.clone()
        } else {
            assets.home.button.clone()
        }),
        hovered: cozy_chat_button_image(assets.home.purple_button_compact.clone()),
        action: auxiliary.action.clone(),
        label: Some(auxiliary.label),
        font_size: 10.0,
        color: if auxiliary.highlighted && enabled {
            TEXT
        } else {
            MUTED
        },
        icon: None,
    }
    .mount(commands, panel, &assets.font);
}
