//! 麻将按钮动作到网络命令的转换。

use super::*;
use leocard_protocol::{ClientCommand, GameCommand, MahjongCommand};

pub fn handle_mahjong_button(
    action: &UiAction,
    client: &mut Option<ResMut<ClientResource>>,
) -> bool {
    let command = match action {
        UiAction::UpdateMahjongRules(rules) => MahjongCommand::UpdateRules { rules: *rules },
        UiAction::MahjongDiscard(tile) => MahjongCommand::Discard { tile: *tile },
        UiAction::MahjongRespond(claim) => MahjongCommand::RespondToClaim { claim: *claim },
        UiAction::MahjongSelfDraw => MahjongCommand::DeclareSelfDraw,
        UiAction::MahjongConcealedKong(tile) => {
            MahjongCommand::DeclareConcealedKong { tile: *tile }
        }
        UiAction::MahjongAddedKong(tile) => MahjongCommand::DeclareAddedKong { tile: *tile },
        _ => return false,
    };
    if let Some(client) = client.as_deref_mut() {
        client
            .0
            .send(ClientCommand::Game(GameCommand::Mahjong(command)));
    }
    true
}
