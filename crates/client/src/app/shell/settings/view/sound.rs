//! Sound volume and game voice switches.

use crate::app::presentation::{TEXT, TableAppearanceSetting, UiComposer, add_text, spawn_node};
use crate::app::shell::SettingsUiAction;

use super::slider::SettingsSlider;
use super::toggle::add_settings_toggle;
use crate::app::runtime::{AppearancePreferences, UiAssets};
use bevy::prelude::*;

pub(super) fn render_sound_settings(
    commands: &mut UiComposer,
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
        add_settings_toggle(commands, row, label, selected, action, assets);
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
    add_settings_toggle(
        commands,
        texas_row,
        "行动语音",
        form.texas_action_voices,
        SettingsUiAction::ToggleTexasActionVoices,
        assets,
    );
}
