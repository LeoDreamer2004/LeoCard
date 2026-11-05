use super::state::MahjongLinkUiState;
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
    Clear,
    SetDistance(u8),
    Act(MahjongLinkAction),
}

impl DomainUiAction for MahjongLinkUiAction {
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
        match self {
            Self::Clear => context.ui.selected = None,
            Self::SetDistance(distance) => context.ui.distance = *distance,
            Self::Act(action) => send_game_command(
                &mut context.client,
                MahjongLinkCommand::Act { action: *action },
            ),
            Self::Select(cell) => {
                let Some(game) = context
                    .client
                    .as_deref()
                    .and_then(|client| client.0.model().game_snapshot())
                    .and_then(GameSnapshot::mahjong_link)
                else {
                    return;
                };
                if game.current_player != Some(game.you)
                    || game.match_complete
                    || game
                        .players
                        .iter()
                        .any(|player| player.player.id == game.you && player.auto_play)
                {
                    return;
                }
                if game.board.tile(*cell).is_none() {
                    return;
                }
                let previous = context.ui.selected;
                if previous == Some(*cell) {
                    context.ui.selected = None;
                } else if let Some(first) =
                    previous.filter(|first| game.board.tile(*first) == game.board.tile(*cell))
                {
                    // The host validates the actual path and preserves the turn on rejection.
                    send_game_command(
                        &mut context.client,
                        MahjongLinkCommand::Act {
                            action: MahjongLinkAction::Match {
                                first,
                                second: *cell,
                            },
                        },
                    );
                    context.ui.selected = None;
                } else {
                    context.ui.selected = Some(*cell);
                    context.ui.distance = 1;
                }
            }
        }
    }
}
