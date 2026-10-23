use super::super::{MinecraftGuessUiAction, MinecraftGuessUiState};
use super::rules::render_rule_rows;
use crate::app::presentation::{UiComposer, spawn_node};
use crate::app::runtime::{AvatarImages, ClientResource, UiAssets};
use crate::app::shell::{
    CozyDropdown, CozyDropdownOption, LobbyPage, LobbyPageStyle, LobbyPlayerSection, UiAction,
    add_lobby_rules_heading,
};
use bevy::prelude::*;
use leocard_minecraft_guess::{GuessMode, MinecraftGuessRuleSet};
use leocard_protocol::LobbySnapshot;

pub(crate) fn render_guess_lobby(
    commands: &mut UiComposer,
    root: Entity,
    client: &ClientResource,
    lobby: &LobbySnapshot,
    state: &MinecraftGuessUiState,
    assets: &UiAssets,
    avatars: &AvatarImages,
) {
    let rules = *lobby.rules.minecraft_guess().expect("guessing lobby rules");
    let page = LobbyPage::spawn(
        commands,
        root,
        client,
        lobby,
        assets,
        LobbyPageStyle::default(),
    );
    let heading = spawn_node(
        commands,
        page.rules,
        Node {
            width: percent(100),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::SpaceBetween,
            column_gap: px(12),
            ..default()
        },
        None,
    );
    add_lobby_rules_heading(commands, heading, "竞猜配置", assets);
    CozyDropdown {
        label: rules.mode.label(),
        toggle_action: UiAction::MinecraftGuess(MinecraftGuessUiAction::ToggleModeMenu),
        close_action: UiAction::MinecraftGuess(MinecraftGuessUiAction::CloseModeMenu),
        options: [GuessMode::TurnBased, GuessMode::Race, GuessMode::Graph]
            .into_iter()
            .map(|mode| CozyDropdownOption {
                label: mode.label(),
                selected: mode == rules.mode,
                action: UiAction::MinecraftGuess(MinecraftGuessUiAction::UpdateRules(
                    MinecraftGuessRuleSet { mode, ..rules },
                )),
            })
            .collect(),
    }
    .render(
        commands,
        root,
        heading,
        page.can_configure,
        state.mode_menu_open,
        assets,
    );
    render_rule_rows(commands, page.rules, rules, page.can_configure, assets);
    let can_start = page.connected_count >= usize::from(MinecraftGuessRuleSet::MIN_PLAYERS)
        && lobby
            .players
            .iter()
            .filter(|player| player.connected)
            .all(|player| player.ready && player.seat.is_some());
    page.render_players(
        commands,
        LobbyPlayerSection::new(
            client,
            lobby,
            assets,
            avatars,
            MinecraftGuessRuleSet::MAX_PLAYERS,
            can_start,
            "等待玩家中",
        ),
    );
}
