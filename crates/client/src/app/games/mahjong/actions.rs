//! 麻将按钮动作到网络命令的转换。

use super::*;
use bevy::ecs::system::SystemParam;
use leocard_protocol::{ClientCommand, GameCommand, MahjongCommand};

#[derive(SystemParam)]
pub struct MahjongActionContext<'w> {
    client: Option<ResMut<'w, ClientResource>>,
}

pub fn dispatch_mahjong_actions(
    mut actions: MessageReader<PressedUiAction>,
    mut context: MahjongActionContext,
) {
    dispatch_domain_actions::<MahjongUiAction, _>(&mut actions, &mut context);
}

impl UiActionHandler<MahjongActionContext<'_>> for MahjongUiAction {
    fn handle(&self, context: &mut MahjongActionContext<'_>) {
        let command = match self {
            MahjongUiAction::UpdateRules(rules) => MahjongCommand::UpdateRules { rules: *rules },
            MahjongUiAction::Discard(tile) => MahjongCommand::Discard { tile: *tile },
            MahjongUiAction::Respond(claim) => MahjongCommand::RespondToClaim { claim: *claim },
            MahjongUiAction::SelfDraw => MahjongCommand::DeclareSelfDraw,
            MahjongUiAction::ConcealedKong(tile) => {
                MahjongCommand::DeclareConcealedKong { tile: *tile }
            }
            MahjongUiAction::AddedKong(tile) => MahjongCommand::DeclareAddedKong { tile: *tile },
        };
        if let Some(client) = context.client.as_deref_mut() {
            client
                .0
                .send(ClientCommand::Game(GameCommand::Mahjong(command)));
        }
    }
}
