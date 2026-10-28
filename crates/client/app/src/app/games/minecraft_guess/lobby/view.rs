use super::super::{MinecraftGuessUiAction, MinecraftGuessUiState};
use super::rules::render_rule_rows;
use crate::app::runtime::{AvatarImages, ClientResource, UiAssets};
use crate::app::shell::{CozyDropdown, CozyDropdownOption, LobbyPage, LobbyPageConfig, UiAction};
use bevy::prelude::*;
use leocard_minecraft_guess::{GuessMode, MinecraftGuessRuleSet};
use leocard_protocol::LobbySnapshot;
use leocard_ui::UiComposer;

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
        avatars,
        LobbyPageConfig::new("竞猜配置").mode(
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
            },
            state.mode_menu_open,
        ),
    );

    render_rule_rows(commands, page.rules, rules, page.can_configure, assets);
}
