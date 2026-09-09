//! UNO 模式、扩展包与房间规则界面。

mod expansion;
mod mode;
mod rules;

use super::{UnoUiAction, UnoUiState};
use crate::app::presentation::{
    ButtonKind, MUTED, add_action_button, add_section_title, add_text, spawn_node,
};
use crate::app::runtime::{AvatarImages, ClientResource, UiAssets};
use crate::app::shell::{LobbyPage, LobbyPageStyle, LobbyPlayerSection, UiAction};
use bevy::prelude::*;
pub(crate) use expansion::*;
use leocard_protocol::LobbySnapshot;
use leocard_uno::UnoRuleSet;
pub(crate) use mode::*;
use rules::*;

pub(crate) fn render_uno_lobby(
    commands: &mut Commands,
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
        LobbyPageStyle::default(),
    );
    let rules_panel = page.rules;
    let connected_count = page.connected_count;
    let can_configure = page.can_configure;
    let title_row = spawn_node(
        commands,
        rules_panel,
        Node {
            width: percent(100),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::SpaceBetween,
            column_gap: px(12),
            ..default()
        },
        None,
    );
    add_section_title(commands, title_row, mode_title(rules), assets);
    render_uno_mode_dropdown(
        commands,
        root,
        title_row,
        rules,
        can_configure,
        ui.mode_menu_open,
        assets,
    );
    add_text(
        commands,
        rules_panel,
        format!("当前人数 {connected_count}/{}", UnoRuleSet::MAX_PLAYERS),
        13.0,
        MUTED,
        assets,
    );
    render_uno_rule_rows(commands, rules_panel, rules, can_configure, assets);
    add_expansion_button(commands, rules_panel, assets);

    let can_start = connected_count >= usize::from(UnoRuleSet::MIN_PLAYERS)
        && lobby
            .players
            .iter()
            .filter(|player| player.connected)
            .all(|player| player.seat.is_some() && player.ready);
    page.render_players(
        commands,
        LobbyPlayerSection::new(
            client,
            lobby,
            assets,
            avatars,
            UnoRuleSet::MAX_PLAYERS,
            can_start,
            "等待玩家中",
        ),
    );
    if ui.expansion_settings_open {
        render_uno_expansion_settings(commands, root, rules, can_configure, assets);
    }
}

fn mode_title(rules: UnoRuleSet) -> &'static str {
    if rules.is_no_mercy() {
        "No Mercy 配置"
    } else if rules.is_flip() {
        "UNO FLIP 配置"
    } else {
        "UNO 配置"
    }
}

fn add_expansion_button(commands: &mut Commands, parent: Entity, assets: &UiAssets) {
    let button = add_action_button(
        commands,
        parent,
        "扩展包设置",
        UiAction::Uno(UnoUiAction::ToggleExpansionSettings),
        ButtonKind::Secondary,
        assets,
    );
    commands.entity(button).insert(Node {
        width: percent(100),
        min_width: px(0),
        height: px(56),
        padding: UiRect::axes(px(18), px(8)),
        align_items: AlignItems::Center,
        justify_content: JustifyContent::Center,
        ..default()
    });
}
