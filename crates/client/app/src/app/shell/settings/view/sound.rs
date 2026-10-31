//! Sound volume and game voice switches.

use super::slider::SettingsSlider;
use super::toggle::add_settings_toggle;
use crate::app::presentation::{TEXT, TableAppearanceSetting, add_text};
use crate::app::runtime::{AppearancePreferences, UiAssets};
use crate::app::shell::SettingsUiAction;
use bevy::prelude::*;
use leocard_ui::{UiComposer, spawn_node};

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
            flex_wrap: FlexWrap::Wrap,
            row_gap: px(8),
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
        (
            "报牌语音",
            form.mahjong_tile_voices,
            SettingsUiAction::ToggleMahjongTileVoices,
        ),
    ] {
        add_settings_toggle(commands, row, label, selected, action, assets);
    }
    for (title, selected, action) in [
        (
            "德州扑克",
            form.texas_action_voices,
            SettingsUiAction::ToggleTexasActionVoices,
        ),
        (
            "七鬼五二三",
            form.qigui_action_voices,
            SettingsUiAction::ToggleQiGuiActionVoices,
        ),
        (
            "斗地主",
            form.doudizhu_action_voices,
            SettingsUiAction::ToggleDoudizhuActionVoices,
        ),
    ] {
        add_text(commands, parent, title, 20.0, TEXT, assets);
        let row = spawn_node(
            commands,
            parent,
            Node {
                width: percent(100),
                flex_direction: FlexDirection::Row,
                ..default()
            },
            None,
        );
        add_settings_toggle(commands, row, "行动语音", selected, action, assets);
    }
}
