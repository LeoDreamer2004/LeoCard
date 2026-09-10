//! 席位、准备、开局与离开房间动作。

use super::super::{
    DomainUiAction, PressedUiAction, UiAction, UiActionHandler, UiState, dispatch_domain_actions,
};
use crate::app::games::uno::UnoUiState;
use crate::app::runtime::ClientResource;
use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use leocard_protocol::{ClientCommand, SeatId};

#[derive(Clone)]
pub(crate) enum LobbyUiAction {
    SelectSeat(SeatId),
    ToggleReady,
    StartGame,
    ReturnToLobby,
    PlayAgain,
    LeaveRoom,
}

impl DomainUiAction for LobbyUiAction {
    fn extract(action: &UiAction) -> Option<&Self> {
        let UiAction::Lobby(action) = action else {
            return None;
        };
        Some(action)
    }
}

#[derive(SystemParam)]
pub(crate) struct LobbyActionContext<'w> {
    client: Option<ResMut<'w, ClientResource>>,
    ui: ResMut<'w, UiState>,
    uno_ui: ResMut<'w, UnoUiState>,
}

pub(crate) fn dispatch_lobby_actions(
    mut actions: MessageReader<PressedUiAction>,
    mut context: LobbyActionContext,
) {
    dispatch_domain_actions::<LobbyUiAction, _>(&mut actions, &mut context);
}

impl UiActionHandler<LobbyActionContext<'_>> for LobbyUiAction {
    fn handle(&self, context: &mut LobbyActionContext<'_>) {
        let client = &mut context.client;
        let ui = &mut context.ui;
        match self {
            LobbyUiAction::SelectSeat(seat) => {
                if let Some(client) = client.as_deref_mut() {
                    client.0.send(ClientCommand::SelectSeat { seat: *seat });
                }
            }
            LobbyUiAction::ToggleReady => toggle_ready(client),
            LobbyUiAction::StartGame => {
                context.uno_ui.expansion_settings_open = false;
                send(client, ClientCommand::StartGame);
            }
            LobbyUiAction::ReturnToLobby => send(client, ClientCommand::ReturnToLobby),
            LobbyUiAction::PlayAgain => send(client, ClientCommand::PlayAgain),
            LobbyUiAction::LeaveRoom => {
                context.uno_ui.expansion_settings_open = false;
                if let Some(client) = client.as_deref_mut() {
                    ui.leaving_room = client.0.send(ClientCommand::LeaveRoom);
                }
            }
        }
    }
}

fn send(client: &mut Option<ResMut<ClientResource>>, command: ClientCommand) {
    if let Some(client) = client.as_deref_mut() {
        client.0.send(command);
    }
}

fn toggle_ready(client: &mut Option<ResMut<ClientResource>>) {
    let Some(client) = client.as_deref_mut() else {
        return;
    };
    let ready = client
        .0
        .model()
        .lobby()
        .and_then(|lobby| {
            let you = client.0.model().you()?;
            lobby.players.iter().find(|player| player.id == you)
        })
        .is_some_and(|player| player.ready);
    client.0.send(ClientCommand::SetReady { ready: !ready });
}
