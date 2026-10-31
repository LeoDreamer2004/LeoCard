use super::LobbyUiAction;
use crate::app::games::game_seat_count;
use crate::app::presentation::{GAME_SIDE_CONTROL_SCALE, LobbySeatHover, LobbySeatLayout, TEXT};
use crate::app::runtime::{ClientResource, UiAssets};
use crate::app::shell::{UiAction, UiState};
use bevy::picking::hover::PickingInteraction;
use bevy::prelude::*;
use leocard_client::ClientPhaseRef;
use leocard_game_common::PlayerId;
use leocard_protocol::{ClientCommand, SeatId};
use leocard_ui::{TextureButton, UiComposer, sliced_image};

#[derive(Component)]
pub(crate) struct LobbyKickMenu {
    player: PlayerId,
    seat: SeatId,
}

pub(crate) fn handle_lobby_seat_right_click(
    mut commands: UiComposer,
    mouse: Res<ButtonInput<MouseButton>>,
    seats: Query<(Entity, &PickingInteraction, &LobbySeatHover)>,
    mut client: Option<ResMut<ClientResource>>,
    assets: Res<UiAssets>,
    mut ui: ResMut<UiState>,
    menus: Query<(Entity, &LobbyKickMenu, &PickingInteraction)>,
) {
    let lobby = client.as_deref().and_then(|client| {
        let model = client.0.model();
        match model.phase() {
            ClientPhaseRef::Lobby(lobby) if model.you().is_some() && model.you() == lobby.host => {
                Some(lobby)
            }
            _ => None,
        }
    });
    let right_click = mouse.just_pressed(MouseButton::Right);
    let clicked_outside = mouse.just_pressed(MouseButton::Left)
        && !menus
            .iter()
            .any(|(_, _, interaction)| *interaction != PickingInteraction::None);
    for (entity, menu, _) in &menus {
        let valid = lobby.is_some_and(|lobby| {
            lobby.players.iter().any(|player| {
                player.player.id == menu.player
                    && player.seat == Some(menu.seat)
                    && player.player.profile_id.0 != [0; 32]
                    && Some(player.player.id) != lobby.host
            })
        });
        if !valid || right_click || clicked_outside {
            commands.entity(entity).despawn();
        }
    }
    if !right_click {
        return;
    }
    let Some(lobby) = lobby else { return };
    let Some((anchor, _, hover)) = seats
        .iter()
        .find(|(_, interaction, _)| **interaction != PickingInteraction::None)
    else {
        return;
    };
    let seat = SeatId(hover.seat);
    let target = lobby
        .players
        .iter()
        .find(|player| player.seat == Some(seat));
    match target {
        Some(player) if Some(player.player.id) == lobby.host => {}
        Some(player) if player.player.profile_id.0 != [0; 32] => {
            ui.social.interaction_menu_open = None;
            let size = Vec2::new(80.0, 36.0) * GAME_SIDE_CONTROL_SCALE;
            let offset =
                LobbySeatLayout::new(game_seat_count(&lobby.rules)).outward_popup(seat.0, size);
            let button = TextureButton {
                node: Node {
                    position_type: PositionType::Absolute,
                    left: px(offset.x),
                    top: px(offset.y),
                    width: px(size.x),
                    height: px(size.y),
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    ..default()
                },
                normal: sliced_image(
                    assets.home.danger_button.clone(),
                    32.0,
                    0.55 * GAME_SIDE_CONTROL_SCALE,
                ),
                hovered: sliced_image(
                    assets.home.danger_button_hover.clone(),
                    32.0,
                    0.55 * GAME_SIDE_CONTROL_SCALE,
                ),
                action: Some(UiAction::Lobby(LobbyUiAction::KickPlayer(player.player.id))),
                label: Some("踢出"),
                font_size: 14.0 * GAME_SIDE_CONTROL_SCALE,
                color: TEXT,
                icon: None,
            }
            .mount(&mut commands, anchor, &assets.font)
            .0;
            commands.entity(button).insert((
                LobbyKickMenu {
                    player: player.player.id,
                    seat,
                },
                GlobalZIndex(1600),
            ));
        }
        _ => {
            let occupied = target.is_none();
            if let Some(client) = client.as_deref_mut() {
                client
                    .0
                    .send(ClientCommand::ConfigureBotSeat { seat, occupied });
            }
        }
    }
}
