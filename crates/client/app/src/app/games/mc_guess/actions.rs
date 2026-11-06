use super::graph::{GraphGuideState, GraphLog};
use super::state::McGuessUiState;
use crate::app::runtime::ClientResource;
use crate::app::shell::{
    DomainUiAction, PressedUiAction, UiAction, UiActionHandler, dispatch_domain_actions,
    game_command,
};
use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use leocard_mc_guess::McGuessRuleSet;
use leocard_protocol::{ClientCommand, GameSnapshot, McGuessCommand};

#[derive(Clone)]
pub(crate) enum McGuessUiAction {
    UpdateRules(McGuessRuleSet),
    ToggleModeMenu,
    CloseModeMenu,
    ToggleGraphGuide,
    CloseGraphGuide,
    ToggleGraphLog,
    Guess(Option<String>),
    Skip,
    Again,
    Lobby,
    DeveloperActivateAll,
}

impl DomainUiAction for McGuessUiAction {
    fn extract(action: &UiAction) -> Option<&Self> {
        let UiAction::McGuess(action) = action else {
            return None;
        };
        Some(action)
    }

    fn rebuilds_ui(&self) -> bool {
        matches!(
            self,
            Self::UpdateRules(_) | Self::ToggleModeMenu | Self::CloseModeMenu
        )
    }
}

#[derive(SystemParam)]
pub(super) struct ActionContext<'w> {
    client: Option<ResMut<'w, ClientResource>>,
    state: ResMut<'w, McGuessUiState>,
    guide: ResMut<'w, GraphGuideState>,
    log: ResMut<'w, GraphLog>,
}

pub(super) fn dispatch_guess_actions(
    mut actions: MessageReader<PressedUiAction>,
    mut context: ActionContext,
) {
    dispatch_domain_actions::<McGuessUiAction, _>(&mut actions, &mut context);
}

impl UiActionHandler<ActionContext<'_>> for McGuessUiAction {
    fn handle(&self, context: &mut ActionContext<'_>) {
        let Some(client) = context.client.as_deref_mut() else {
            return;
        };
        let state = &mut context.state;
        let guide = &mut context.guide;
        let log = &mut context.log;
        let command = match self {
            McGuessUiAction::ToggleGraphLog => {
                log.toggle();
                return;
            }
            McGuessUiAction::ToggleGraphGuide => {
                guide.toggle();
                return;
            }
            McGuessUiAction::CloseGraphGuide => {
                guide.close();
                return;
            }
            McGuessUiAction::ToggleModeMenu => {
                state.mode_menu_open = !state.mode_menu_open;
                return;
            }
            McGuessUiAction::CloseModeMenu => {
                state.mode_menu_open = false;
                return;
            }
            McGuessUiAction::UpdateRules(rules) => {
                state.mode_menu_open = false;
                game_command(McGuessCommand::UpdateRules { rules: *rules })
            }
            McGuessUiAction::Guess(entry) => {
                if !client
                    .0
                    .model()
                    .game_snapshot()
                    .and_then(GameSnapshot::mc_guess)
                    .is_some_and(|game| state.can_guess(game))
                {
                    return;
                }
                game_command(McGuessCommand::Guess {
                    entry: entry.as_ref().unwrap_or(&state.input).clone(),
                })
            }
            McGuessUiAction::Skip => game_command(McGuessCommand::Skip),
            McGuessUiAction::Again => ClientCommand::PlayAgain,
            McGuessUiAction::Lobby => ClientCommand::ReturnToLobby,
            McGuessUiAction::DeveloperActivateAll => {
                if !(cfg!(debug_assertions) || cfg!(feature = "developer")) {
                    return;
                }
                game_command(McGuessCommand::DeveloperActivateAll)
            }
        };
        client.0.send(command);
    }
}
