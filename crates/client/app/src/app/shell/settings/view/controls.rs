use super::toggle::add_settings_toggle;
use crate::app::presentation::{TEXT, add_title, rule_help_style};
use crate::app::runtime::{AppearancePreferences, UiAssets};
use crate::app::shell::{SettingsUiAction, ShellOverlayLayer};
use bevy::prelude::*;
use leocard_ui::{UiComposer, add_help_tooltip, spawn_node};

pub(super) fn render_control_settings(
    commands: &mut UiComposer,
    parent: Entity,
    form: &AppearancePreferences,
    assets: &UiAssets,
) {
    add_title(commands, parent, "麻将", 20.0, TEXT, assets);
    let row = spawn_node(
        commands,
        parent,
        Node {
            width: percent(100),
            align_items: AlignItems::Center,
            column_gap: px(6),
            ..default()
        },
        None,
    );
    let toggle = add_settings_toggle(
        commands,
        row,
        "右键摸打",
        form.controls.mahjong_right_click_draw_discard,
        SettingsUiAction::ToggleMahjongRightClickDrawDiscard,
        assets,
    );
    commands
        .entity(toggle)
        .entry::<Node>()
        .and_modify(|mut node| node.flex_grow = 0.0);
    let tooltip = add_help_tooltip(
        commands,
        row,
        "开启后，在牌局任意位置右键可打出刚摸入的牌。",
        rule_help_style(assets),
    );
    commands
        .entity(tooltip)
        .insert(GlobalZIndex(ShellOverlayLayer::Dialog.z_index().0 + 1));
}
