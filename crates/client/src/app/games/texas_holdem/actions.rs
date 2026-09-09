//! 德州扑克按钮动作到本地状态或网络命令的转换。

use super::TexasHoldemUiState;
use crate::app::runtime::ClientResource;
use crate::app::shell::{
    DomainUiAction, PressedUiAction, UiAction, UiActionHandler, dispatch_domain_actions,
    send_game_command,
};
use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use leocard_protocol::TexasHoldemCommand;
use leocard_texas_holdem::{TexasHoldemAction, TexasHoldemRuleSet};

#[derive(Clone)]
pub(crate) enum TexasHoldemUiAction {
    UpdateRules(TexasHoldemRuleSet),
    SetRaiseTo(u32),
    Act(TexasHoldemAction),
}

impl DomainUiAction for TexasHoldemUiAction {
    fn extract(action: &UiAction) -> Option<&Self> {
        let UiAction::TexasHoldem(action) = action else {
            return None;
        };
        Some(action)
    }
}

#[derive(SystemParam)]
pub(crate) struct TexasHoldemActionContext<'w> {
    client: Option<ResMut<'w, ClientResource>>,
    ui: ResMut<'w, TexasHoldemUiState>,
}

pub(super) fn dispatch_texas_holdem_actions(
    mut actions: MessageReader<PressedUiAction>,
    mut context: TexasHoldemActionContext,
) {
    dispatch_domain_actions::<TexasHoldemUiAction, _>(&mut actions, &mut context);
}

impl UiActionHandler<TexasHoldemActionContext<'_>> for TexasHoldemUiAction {
    fn handle(&self, context: &mut TexasHoldemActionContext<'_>) {
        match self {
            TexasHoldemUiAction::UpdateRules(rules) => {
                send_game_command(
                    &mut context.client,
                    TexasHoldemCommand::UpdateRules { rules: *rules },
                );
            }
            TexasHoldemUiAction::SetRaiseTo(target) => context.ui.raise_to = *target,
            TexasHoldemUiAction::Act(action) => {
                send_game_command(
                    &mut context.client,
                    TexasHoldemCommand::Act { action: *action },
                );
            }
        }
    }
}
