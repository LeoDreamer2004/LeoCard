use super::BlackjackUiState;
use crate::app::runtime::ClientResource;
use crate::app::shell::{
    DomainUiAction, PressedUiAction, UiAction, UiActionHandler, dispatch_domain_actions,
    send_game_command,
};
use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use leocard_blackjack::{BlackjackAction, BlackjackRuleSet};
use leocard_protocol::BlackjackCommand;

#[derive(Clone)]
pub(crate) enum BlackjackUiAction {
    UpdateRules(BlackjackRuleSet),
    SetBet(u32),
    Act(BlackjackAction),
    ToggleGuide,
}

impl DomainUiAction for BlackjackUiAction {
    fn extract(action: &UiAction) -> Option<&Self> {
        let UiAction::Blackjack(action) = action else {
            return None;
        };
        Some(action)
    }
}

#[derive(SystemParam)]
pub(super) struct BlackjackActionContext<'w> {
    client: Option<ResMut<'w, ClientResource>>,
    ui: ResMut<'w, BlackjackUiState>,
}

pub(super) fn dispatch_blackjack_actions(
    mut actions: MessageReader<PressedUiAction>,
    mut context: BlackjackActionContext,
) {
    dispatch_domain_actions::<BlackjackUiAction, _>(&mut actions, &mut context);
}

impl UiActionHandler<BlackjackActionContext<'_>> for BlackjackUiAction {
    fn handle(&self, context: &mut BlackjackActionContext<'_>) {
        match self {
            Self::SetBet(amount) => context.ui.bet = *amount,
            Self::ToggleGuide => context.ui.guide_open = !context.ui.guide_open,
            Self::Act(action) => send_game_command(
                &mut context.client,
                BlackjackCommand::Act { action: *action },
            ),
            Self::UpdateRules(rules) => send_game_command(
                &mut context.client,
                BlackjackCommand::UpdateRules { rules: *rules },
            ),
        }
    }
}
