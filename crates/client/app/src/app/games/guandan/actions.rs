use super::state::GuandanUiState;
use crate::app::runtime::ClientResource;
use crate::app::shell::{
    DomainUiAction, PressedUiAction, UiAction, UiActionHandler, dispatch_domain_actions,
    send_game_command,
};
use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use leocard_client::{PlayerEconomy, PropId};
use leocard_guandan::{GuandanAction, GuandanRuleSet, GuandanTributeAction};
use leocard_protocol::{GameSnapshot, GuandanCommand};

#[derive(Clone)]
pub(crate) enum GuandanUiAction {
    UpdateRules(GuandanRuleSet),
    ToggleCard,
    Play,
    Pass,
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
                    context.ui.selected.clear();
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
                if context.ui.selected.len() != 1 {
                    return;
                }
                let Some(card) = context.ui.selected.iter().copied().next() else {
                    return;
                };
                if game.received_tribute == Some(card) {
                    return;
                }
                let action = match game.tribute_action {
                    Some(GuandanTributeAction::Tribute) => GuandanAction::Tribute(card),
                    Some(GuandanTributeAction::ReturnTribute) => GuandanAction::ReturnTribute(card),
                    None => return,
                };
                send_game_command(&mut context.client, GuandanCommand::Act(action));
                context.ui.selected.clear();
            }
        }
    }
}
