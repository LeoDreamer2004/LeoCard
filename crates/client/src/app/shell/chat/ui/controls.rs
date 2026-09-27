use super::super::*;
use super::panel::ChatAuxiliaryAction;
use super::panel::{add_chat_button_highlight, cozy_chat_button_image};
use crate::app::presentation::{MUTED, TEXT, add_text};
use crate::app::runtime::UiAssets;
use crate::app::shell::{ChatUiAction, SocialUiAction, UiAction};
use bevy::prelude::*;
use bevy::ui::FocusPolicy;

pub(super) fn add_chat_toggle(
    commands: &mut Commands,
    panel: Entity,
    chat: &ChatPanelState,
    assets: &UiAssets,
) {
    let toggle = commands
        .spawn((
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
        ))
        .id();
    commands.entity(panel).add_child(toggle);
}

pub(super) fn add_auto_play_toggle(
    commands: &mut Commands,
    panel: Entity,
    enabled: bool,
    assets: &UiAssets,
) {
    let auto_button = commands
        .spawn((
            Button,
            UiAction::Social(SocialUiAction::ToggleAutoPlay),
            Node {
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
            cozy_chat_button_image(if enabled {
                assets.home.purple_button_compact.clone()
            } else {
                assets.home.button.clone()
            }),
        ))
        .id();
    commands.entity(panel).add_child(auto_button);
    add_chat_button_highlight(commands, auto_button, assets);
    let icon = commands
        .spawn((
            Node {
                width: px(25),
                height: px(25),
                ..default()
            },
            ImageNode::new(assets.controls.robot_icon.clone()),
            FocusPolicy::Pass,
        ))
        .id();
    commands.entity(auto_button).add_child(icon);
}

pub(super) fn add_auxiliary_actions(
    commands: &mut Commands,
    panel: Entity,
    actions: &[ChatAuxiliaryAction],
    assets: &UiAssets,
) {
    for (index, action) in actions.iter().enumerate() {
        add_auxiliary_action(commands, panel, assets, action, index);
    }
}

fn add_auxiliary_action(
    commands: &mut Commands,
    panel: Entity,
    assets: &UiAssets,
    auxiliary: &ChatAuxiliaryAction,
    index: usize,
) {
    let enabled = auxiliary.action.is_some();
    let button = commands
        .spawn((
            Node {
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
            cozy_chat_button_image(if auxiliary.highlighted && enabled {
                assets.home.purple_button_compact.clone()
            } else {
                assets.home.button.clone()
            }),
        ))
        .id();
    if let Some(action) = auxiliary.action.clone() {
        commands.entity(button).insert((Button, action));
    }
    commands.entity(panel).add_child(button);
    if enabled {
        add_chat_button_highlight(commands, button, assets);
    }
    let label = add_text(
        commands,
        button,
        auxiliary.label,
        10.0,
        if auxiliary.highlighted && enabled {
            TEXT
        } else {
            MUTED
        },
        assets,
    );
    commands.entity(label).insert(FocusPolicy::Pass);
}
