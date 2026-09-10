use super::super::*;
use super::panel::ChatAuxiliaryAction;
use crate::app::presentation::{ACCENT, BORDER, ButtonTint, MUTED, add_text};
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
            ButtonTint {
                normal: Color::WHITE,
                hovered: Color::srgb(1.0, 0.90, 0.56),
                pressed: Color::srgb(0.68, 0.82, 0.90),
            },
            Node {
                position_type: PositionType::Absolute,
                left: px(-32),
                top: px(154),
                width: px(32),
                min_width: px(32),
                max_width: px(32),
                height: px(32),
                min_height: px(32),
                max_height: px(32),
                flex_shrink: 0.0,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                border_radius: BorderRadius::all(px(7)),
                ..default()
            },
            ImageNode::new(if chat.open {
                assets.social.chat_close_icon.clone()
            } else {
                assets.social.chat_open_icon.clone()
            })
            .with_color(Color::WHITE),
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
    let normal = if enabled {
        Color::srgb(0.18, 0.68, 0.38)
    } else {
        Color::srgb(0.13, 0.39, 0.29)
    };
    let auto_button = commands
        .spawn((
            Button,
            UiAction::Social(SocialUiAction::ToggleAutoPlay),
            ButtonTint {
                normal,
                hovered: if enabled {
                    Color::srgb(0.43, 0.94, 0.60)
                } else {
                    Color::srgb(0.12, 0.38, 0.27)
                },
                pressed: Color::srgb(0.11, 0.27, 0.21),
            },
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
                border: UiRect::all(px(1)),
                border_radius: BorderRadius::all(px(7)),
                ..default()
            },
            BorderColor::all(if enabled { ACCENT } else { BORDER }),
            ImageNode::new(assets.controls.secondary_button.clone())
                .with_mode(NodeImageMode::Stretch)
                .with_color(normal),
        ))
        .id();
    commands.entity(panel).add_child(auto_button);
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
    let normal = if enabled {
        Color::srgb(0.13, 0.39, 0.29)
    } else {
        Color::srgb(0.09, 0.16, 0.14)
    };
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
                border: UiRect::all(px(1)),
                border_radius: BorderRadius::all(px(7)),
                ..default()
            },
            BorderColor::all(if auxiliary.highlighted && enabled {
                ACCENT
            } else {
                BORDER
            }),
            ImageNode::new(assets.controls.secondary_button.clone())
                .with_mode(NodeImageMode::Stretch)
                .with_color(normal),
        ))
        .id();
    if let Some(action) = auxiliary.action.clone() {
        commands.entity(button).insert((
            Button,
            action,
            ButtonTint {
                normal,
                hovered: Color::srgb(0.22, 0.56, 0.39),
                pressed: Color::srgb(0.10, 0.27, 0.20),
            },
        ));
    }
    commands.entity(panel).add_child(button);
    let label = add_text(
        commands,
        button,
        auxiliary.label,
        10.0,
        if auxiliary.highlighted && enabled {
            ACCENT
        } else {
            MUTED
        },
        assets,
    );
    commands.entity(label).insert(FocusPolicy::Pass);
}
