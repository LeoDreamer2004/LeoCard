//! 席位、准备、开局与离开房间动作。

use super::*;
use leocard_protocol::ClientCommand;

pub fn handle_lobby_button(
    action: &UiAction,
    client: &mut Option<ResMut<ClientResource>>,
    ui: &mut UiState,
) -> bool {
    match action {
        UiAction::SelectSeat(seat) => {
            if let Some(client) = client.as_deref_mut() {
                client.0.send(ClientCommand::SelectSeat { seat: *seat });
            }
        }
        UiAction::ToggleReady => toggle_ready(client),
        UiAction::StartGame => {
            ui.uno.expansion_settings_open = false;
            send(client, ClientCommand::StartGame);
        }
        UiAction::ReturnToLobby => send(client, ClientCommand::ReturnToLobby),
        UiAction::PlayAgain => send(client, ClientCommand::PlayAgain),
        UiAction::LeaveRoom => {
            ui.uno.expansion_settings_open = false;
            if let Some(client) = client.as_deref_mut() {
                ui.leaving_room = client.0.send(ClientCommand::LeaveRoom);
            }
        }
        _ => return false,
    }
    true
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
