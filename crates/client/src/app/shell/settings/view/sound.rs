//! Sound volume and game voice switches.

use crate::app::shell::SettingsUiAction;
use bevy::picking::Pickable;
use bevy::ui_widgets::Button;

use super::super::super::UiAction;
use super::super::SettingsVoiceToggle;
use super::slider::SettingsSlider;
use crate::app::presentation::{TEXT, TableAppearanceSetting, add_text, spawn_node};
use crate::app::runtime::{AppearancePreferences, UiAssets};
use bevy::prelude::*;

pub(super) fn render_sound_settings(
    commands: &mut Commands,
    parent: Entity,
    form: &AppearancePreferences,
    assets: &UiAssets,
) {
    SettingsSlider::new(form, assets).render(commands, parent, TableAppearanceSetting::Volume);
    spawn_node(
        commands,
        parent,
        Node {
            width: percent(100),
            height: px(1),
            margin: UiRect::vertical(px(8)),
            ..default()
        },
        Some(Color::srgba(0.70, 0.68, 0.78, 0.36)),
    );
    add_text(commands, parent, "麻将", 20.0, TEXT, assets);
    let row = spawn_node(
        commands,
        parent,
        Node {
            width: percent(100),
            flex_direction: FlexDirection::Row,
            column_gap: px(18),
            ..default()
        },
        None,
    );
    for (label, selected, action) in [
        (
            "吃碰语音",
            form.mahjong_action_voices,
            SettingsUiAction::ToggleMahjongActionVoices,
        ),
        (
            "报番语音",
            form.mahjong_fan_voices,
            SettingsUiAction::ToggleMahjongFanVoices,
        ),
    ] {
        add_voice_toggle(commands, row, label, selected, action, assets);
    }
    add_text(commands, parent, "德州扑克", 20.0, TEXT, assets);
    let texas_row = spawn_node(
        commands,
        parent,
        Node {
            width: percent(100),
            flex_direction: FlexDirection::Row,
            ..default()
        },
        None,
    );
    add_voice_toggle(
        commands,
        texas_row,
        "行动语音",
        form.texas_action_voices,
        SettingsUiAction::ToggleTexasActionVoices,
        assets,
    );
}

fn add_voice_toggle(
    commands: &mut Commands,
    parent: Entity,
    label: &str,
    selected: bool,
    action: SettingsUiAction,
    assets: &UiAssets,
) {
    let button = commands
        .spawn((
            Button,
            UiAction::Settings(action),
            Node {
                flex_grow: 1.0,
                min_width: px(0),
                height: px(48),
                align_items: AlignItems::Center,
                column_gap: px(8),
                ..default()
            },
        ))
        .id();
    commands.entity(parent).add_child(button);
    let icon = commands
        .spawn((
            Node {
                width: px(34),
                height: px(34),
                ..default()
            },
            ImageNode::new(if selected {
                assets.home.checkbox_selected.clone()
            } else {
                assets.home.checkbox.clone()
            }),
            Pickable::IGNORE,
        ))
        .id();
    commands.entity(button).add_child(icon);
    commands
        .entity(button)
        .insert(SettingsVoiceToggle { icon, selected });
    add_text(commands, button, label, 16.0, TEXT, assets);
}
