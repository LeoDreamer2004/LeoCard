//! UNO 模式、扩展包与房间规则界面。

use super::super::{UnoUiAction, UnoUiState};
use super::mode::uno_mode_dropdown;
use super::rules::*;
use super::*;
use crate::app::runtime::{AvatarImages, ClientResource, UiAssets};
use crate::app::shell::{LobbyPage, LobbyPageConfig, UiAction, add_cozy_button};
use bevy::prelude::*;
use leocard_protocol::LobbySnapshot;
use leocard_ui::UiComposer;
use leocard_uno::UnoRuleSet;

pub(crate) fn render_uno_lobby(
    commands: &mut UiComposer,
    root: Entity,
    client: &ClientResource,
    lobby: &LobbySnapshot,
    ui: &UnoUiState,
    assets: &UiAssets,
    avatars: &AvatarImages,
) {
    let rules = *lobby.rules.uno().expect("UNO 大厅应携带对应规则");
    let page = LobbyPage::spawn(
        commands,
        root,
        client,
        lobby,
        assets,
        avatars,
        LobbyPageConfig::new(mode_title(rules)).mode(uno_mode_dropdown(rules), ui.mode_menu_open),
    );
    let rules_panel = page.rules;
    let can_configure = page.can_configure;

    render_uno_rule_rows(commands, rules_panel, rules, can_configure, assets);
    add_expansion_button(commands, rules_panel, assets);

    if ui.expansion_settings_open || ui.expansion_settings_progress > 0.0 {
        render_uno_expansion_settings(
            commands,
            root,
            rules,
            can_configure,
            ui.expansion_settings_progress,
            assets,
        );
    }
}

pub(super) fn mode_title(rules: UnoRuleSet) -> &'static str {
    if rules.is_no_mercy() {
        "No Mercy 配置"
    } else if rules.is_flip() {
        "UNO FLIP 配置"
    } else {
        "UNO 配置"
    }
}

pub(super) fn add_expansion_button(commands: &mut UiComposer, parent: Entity, assets: &UiAssets) {
    add_cozy_button(
        commands,
        parent,
        "扩展包设置",
        UiAction::Uno(UnoUiAction::ToggleExpansionSettings),
        assets,
        percent(100),
        48.0,
    );
}
