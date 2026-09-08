//! 席位、准备、开局与离开房间动作。

use super::*;
use bevy::ecs::system::SystemParam;
use leocard_protocol::ClientCommand;

#[derive(SystemParam)]
pub struct LobbyActionContext<'w> {
    client: Option<ResMut<'w, ClientResource>>,
    ui: ResMut<'w, UiState>,
}

pub fn dispatch_lobby_actions(
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
                ui.uno.expansion_settings_open = false;
                send(client, ClientCommand::StartGame);
            }
            LobbyUiAction::ReturnToLobby => send(client, ClientCommand::ReturnToLobby),
            LobbyUiAction::PlayAgain => send(client, ClientCommand::PlayAgain),
            LobbyUiAction::LeaveRoom => {
                ui.uno.expansion_settings_open = false;
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
