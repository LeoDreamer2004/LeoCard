use super::DoudizhuUiState;
use crate::app::runtime::{ClientResource, HostRulePreferences};
use crate::app::shell::{
    DomainUiAction, PressedUiAction, UiAction, UiActionHandler, dispatch_domain_actions,
    send_game_command,
};
use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use leocard_doudizhu::{Action, Mode, RuleSet, hint_options};
use leocard_protocol::DoudizhuCommand;

#[derive(Clone)]
pub(crate) enum DoudizhuUiAction {
    UpdateRules(RuleSet),
    SelectMode(Mode),
    ToggleModeMenu,
    CloseModeMenu,
    ToggleCard,
    Hint,
    Play,
    Act(Action),
    ToggleKitty,
    ShowPreviousTrick,
    ToggleCounter,
    ToggleCounterDrawer,
}

impl DomainUiAction for DoudizhuUiAction {
    fn extract(action: &UiAction) -> Option<&Self> {
        if let UiAction::Doudizhu(action) = action {
            Some(action)
        } else {
            None
        }
    }

    fn rebuilds_ui(&self) -> bool {
        !matches!(self, Self::ToggleCard)
    }
}

#[derive(SystemParam)]
pub(super) struct ActionContext<'w> {
    client: Option<ResMut<'w, ClientResource>>,
    ui: ResMut<'w, DoudizhuUiState>,
    preferences: Res<'w, HostRulePreferences>,
}

pub(super) fn dispatch_actions(
    mut actions: MessageReader<PressedUiAction>,
    mut context: ActionContext,
) {
    dispatch_domain_actions::<DoudizhuUiAction, _>(&mut actions, &mut context);
}

impl UiActionHandler<ActionContext<'_>> for DoudizhuUiAction {
    fn handle(&self, context: &mut ActionContext<'_>) {
        match self {
            Self::ToggleModeMenu => context.ui.mode_menu_open = !context.ui.mode_menu_open,
            Self::CloseModeMenu => context.ui.mode_menu_open = false,
            Self::SelectMode(mode) => {
                context.ui.mode_menu_open = false;
                send_game_command(
                    &mut context.client,
                    DoudizhuCommand::UpdateRules {
                        rules: context.preferences.doudizhu_modes[*mode as usize],
                    },
                );
            }
            Self::UpdateRules(rules) => send_game_command(
                &mut context.client,
                DoudizhuCommand::UpdateRules { rules: *rules },
            ),
            Self::Act(action) => {
                send_game_command(&mut context.client, DoudizhuCommand::Act(action.clone()))
            }
            Self::ToggleCard => {}
            Self::Play => send_game_command(
                &mut context.client,
                DoudizhuCommand::Act(Action::Play(context.ui.selected.iter().copied().collect())),
            ),
            Self::Hint => {
                if let Some(game) = context
                    .client
                    .as_deref()
                    .and_then(|c| c.0.model().doudizhu_game())
                {
                    let suggestions = hint_options(
                        &game.your_hand,
                        game.table.as_ref().map(|(_, p)| p),
                        game.rules,
                    );
                    let next = suggestions
                        .iter()
                        .position(|play| {
                            play.cards.len() == context.ui.selected.len()
                                && play
                                    .cards
                                    .iter()
                                    .all(|card| context.ui.selected.contains(card))
                        })
                        .map_or(0, |index| (index + 1) % suggestions.len());
                    context.ui.selected = suggestions
                        .get(next)
                        .map(|play| play.cards.iter().copied().collect())
                        .unwrap_or_default();
                }
            }
            Self::ToggleKitty => context.ui.kitty_open = !context.ui.kitty_open,
            Self::ShowPreviousTrick => context.ui.history.reveal(),
            Self::ToggleCounter => context.ui.counter.enabled = !context.ui.counter.enabled,
            Self::ToggleCounterDrawer => {
                context.ui.counter.drawer_open = !context.ui.counter.drawer_open
            }
        }
    }
}
