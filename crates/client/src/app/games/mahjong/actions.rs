//! 麻将按钮动作到网络命令的转换。

use crate::app::runtime::ClientResource;
use crate::app::shell::{
    DomainUiAction, PressedUiAction, UiAction, UiActionHandler, dispatch_domain_actions,
    send_game_command,
};
use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use leocard_mahjong::{MahjongClaim, MahjongRuleSet, MahjongTile, MahjongTileKind};
use leocard_protocol::MahjongCommand;

#[derive(Clone)]
pub(crate) enum MahjongUiAction {
    UpdateRules(MahjongRuleSet),
    Discard(MahjongTile),
    Respond(MahjongClaim),
    SelfDraw,
    ConcealedKong(MahjongTileKind),
    AddedKong(MahjongTile),
}

impl DomainUiAction for MahjongUiAction {
    fn extract(action: &UiAction) -> Option<&Self> {
        let UiAction::Mahjong(action) = action else {
            return None;
        };
        Some(action)
    }
}

#[derive(SystemParam)]
pub(crate) struct MahjongActionContext<'w> {
    client: Option<ResMut<'w, ClientResource>>,
}

pub(super) fn dispatch_mahjong_actions(
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
        send_game_command(&mut context.client, command);
    }
}
