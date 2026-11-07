use super::state::{MahjongLinkUiState, can_act};
use crate::app::runtime::ClientResource;
use crate::app::shell::{
    DomainUiAction, PressedUiAction, UiAction, UiActionHandler, dispatch_domain_actions,
    send_game_command,
};
use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use leocard_mahjong_link::{LinkCell, MahjongLinkAction};
use leocard_protocol::{GameSnapshot, MahjongLinkCommand};

#[derive(Clone)]
pub(crate) enum MahjongLinkUiAction {
    Select(LinkCell),
    Pass,
}

impl DomainUiAction for MahjongLinkUiAction {
    fn rebuilds_ui(&self) -> bool {
        false
    }

    fn extract(action: &UiAction) -> Option<&Self> {
        let UiAction::MahjongLink(action) = action else {
            return None;
        };
        Some(action)
    }
}

#[derive(SystemParam)]
pub(super) struct ActionContext<'w> {
    client: Option<ResMut<'w, ClientResource>>,
    ui: ResMut<'w, MahjongLinkUiState>,
}

pub(super) fn dispatch_actions(
    mut actions: MessageReader<PressedUiAction>,
    mut context: ActionContext,
) {
    dispatch_domain_actions::<MahjongLinkUiAction, _>(&mut actions, &mut context);
}

impl UiActionHandler<ActionContext<'_>> for MahjongLinkUiAction {
    fn handle(&self, context: &mut ActionContext<'_>) {
        let Some(game) = context
            .client
            .as_deref()
            .and_then(|client| client.0.model().game_snapshot())
            .and_then(GameSnapshot::mahjong_link)
        else {
            return;
        };
        if !can_act(game) || context.ui.pending_push.is_some() {
            return;
        }
        let action = match self {
            Self::Pass => Some(MahjongLinkAction::Pass),
            Self::Select(cell) => {
                if game.board.tile(*cell).is_none() {
                    return;
                }
                if context.ui.selected == Some(*cell) {
                    context.ui.selected = None;
                    None
                } else if let Some(first) = context
                    .ui
                    .selected
                    .filter(|first| game.board.tile(*first) == game.board.tile(*cell))
                {
                    Some(MahjongLinkAction::Match {
                        first,
                        second: *cell,
                    })
                } else {
                    context.ui.selected = Some(*cell);
                    None
                }
            }
        };
        if let Some(action) = action {
            send_game_command(&mut context.client, MahjongLinkCommand::Act { action });
            context.ui.selected = None;
            context.ui.drag = None;
        }
    }
}
