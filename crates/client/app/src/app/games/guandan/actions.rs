use super::state::{GuandanUiState, local_turn, tribute_choice};
use crate::app::games::HandSelection;
use crate::app::runtime::ClientResource;
use crate::app::shell::{
    DomainUiAction, PressedUiAction, UiAction, UiActionHandler, dispatch_domain_actions,
    send_game_command,
};
use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use leocard_client::{PlayerEconomy, PropId};
use leocard_guandan::{GuandanAction, GuandanRuleSet};
use leocard_protocol::{GameSnapshot, GuandanCommand};

#[derive(Clone)]
pub(crate) enum GuandanUiAction {
    UpdateRules(GuandanRuleSet),
    ToggleCard,
    Play,
    Pass,
    Hint,
    ShowPreviousTrick,
    SubmitTribute,
    ToggleCounter,
    ToggleCounterDrawer,
}

impl DomainUiAction for GuandanUiAction {
    fn extract(action: &UiAction) -> Option<&Self> {
        let UiAction::Guandan(action) = action else {
            return None;
        };
        Some(action)
    }

    fn rebuilds_ui(&self) -> bool {
        !matches!(self, Self::ToggleCard)
    }
}

#[derive(SystemParam)]
pub(super) struct GuandanActionContext<'w> {
    client: Option<ResMut<'w, ClientResource>>,
    ui: ResMut<'w, GuandanUiState>,
    economy: Res<'w, PlayerEconomy>,
}

pub(super) fn dispatch_actions(
    mut actions: MessageReader<PressedUiAction>,
    mut context: GuandanActionContext,
) {
    dispatch_domain_actions::<GuandanUiAction, _>(&mut actions, &mut context);
}

impl UiActionHandler<GuandanActionContext<'_>> for GuandanUiAction {
    fn handle(&self, context: &mut GuandanActionContext<'_>) {
        match self {
            Self::UpdateRules(rules) => send_game_command(
                &mut context.client,
                GuandanCommand::UpdateRules { rules: *rules },
            ),
            Self::ToggleCard => {}
            Self::Hint => {
                let ui = &mut context.ui;
                if let Some(cards) = ui.hints.next_cards() {
                    let selected = cards.iter().copied().collect();
                    *ui.selected_cards_mut() = selected;
                }
            }
            Self::ShowPreviousTrick => context.ui.previous_reveal = 2.0,
            Self::ToggleCounter => {
                if context.economy.active(PropId::CardCounter) {
                    context.ui.counter.enabled = !context.ui.counter.enabled;
                }
            }
            Self::ToggleCounterDrawer => {
                context.ui.counter.drawer_open = !context.ui.counter.drawer_open;
            }
            Self::Pass => send_game_command(
                &mut context.client,
                GuandanCommand::Act(GuandanAction::Pass),
            ),
            Self::Play => {
                let Some(game) = context.client.as_deref().and_then(|client| {
                    client
                        .0
                        .model()
                        .game_snapshot()
                        .and_then(GameSnapshot::guandan)
                }) else {
                    return;
                };
                let cards = game
                    .your_hand
                    .iter()
                    .copied()
                    .filter(|card| context.ui.selected.contains(card))
                    .collect::<Vec<_>>();
                if !cards.is_empty() {
                    send_game_command(
                        &mut context.client,
                        GuandanCommand::Act(GuandanAction::Play(cards)),
                    );
                    context.ui.clear_selection();
                }
            }
            Self::SubmitTribute => {
                let Some(game) = context.client.as_deref().and_then(|client| {
                    client
                        .0
                        .model()
                        .game_snapshot()
                        .and_then(GameSnapshot::guandan)
                }) else {
                    return;
                };
                if context.ui.selected_cards().len() != 1 {
                    return;
                }
                let Some(card) = context.ui.selected_cards().iter().copied().next() else {
                    return;
                };
                let Some(action) = tribute_choice(game, card).filter(|_| local_turn(game)) else {
                    return;
                };
                send_game_command(&mut context.client, GuandanCommand::Act(action));
                context.ui.clear_selection();
            }
        }
    }
}
