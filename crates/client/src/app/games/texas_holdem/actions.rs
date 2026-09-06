//! 德州扑克按钮动作到本地状态或网络命令的转换。

use super::*;
use leocard_protocol::{ClientCommand, GameCommand, TexasHoldemCommand};

pub fn handle_texas_holdem_button(
    action: &UiAction,
    client: &mut Option<ResMut<ClientResource>>,
    ui: &mut UiState,
) -> bool {
    match action {
        UiAction::UpdateTexasRules(rules) => {
            if let Some(client) = client.as_deref_mut() {
                client.0.send(ClientCommand::Game(GameCommand::TexasHoldem(
                    TexasHoldemCommand::UpdateRules { rules: *rules },
                )));
            }
        }
        UiAction::SetTexasRaiseTo(target) => ui.texas_holdem.raise_to = *target,
        UiAction::TexasAct(action) => {
            if let Some(client) = client.as_deref_mut() {
                client.0.send(ClientCommand::Game(GameCommand::TexasHoldem(
                    TexasHoldemCommand::Act { action: *action },
                )));
            }
        }
        _ => return false,
    }
    true
}
