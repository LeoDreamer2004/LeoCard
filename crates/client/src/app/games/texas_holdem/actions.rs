//! 德州扑克按钮动作到本地状态或网络命令的转换。

use super::*;
use bevy::ecs::system::SystemParam;
use leocard_protocol::{ClientCommand, GameCommand, TexasHoldemCommand};

#[derive(SystemParam)]
pub struct TexasHoldemActionContext<'w> {
    client: Option<ResMut<'w, ClientResource>>,
    ui: ResMut<'w, UiState>,
}

pub fn dispatch_texas_holdem_actions(
    mut actions: MessageReader<PressedUiAction>,
    mut context: TexasHoldemActionContext,
) {
    dispatch_domain_actions::<TexasHoldemUiAction, _>(&mut actions, &mut context);
}

impl UiActionHandler<TexasHoldemActionContext<'_>> for TexasHoldemUiAction {
    fn handle(&self, context: &mut TexasHoldemActionContext<'_>) {
        match self {
            TexasHoldemUiAction::UpdateRules(rules) => {
                if let Some(client) = context.client.as_deref_mut() {
                    client.0.send(ClientCommand::Game(GameCommand::TexasHoldem(
                        TexasHoldemCommand::UpdateRules { rules: *rules },
                    )));
                }
            }
            TexasHoldemUiAction::SetRaiseTo(target) => context.ui.texas_holdem.raise_to = *target,
            TexasHoldemUiAction::Act(action) => {
                if let Some(client) = context.client.as_deref_mut() {
                    client.0.send(ClientCommand::Game(GameCommand::TexasHoldem(
                        TexasHoldemCommand::Act { action: *action },
                    )));
                }
            }
        }
    }
}
