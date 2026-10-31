use super::entrance::LobbyEntrancePlugin;
use super::{animate_lobby_seat_hover, dispatch_lobby_actions, handle_lobby_seat_right_click};
use crate::app::runtime::ClientUpdateSet;
use crate::app::shell::UiActionSet;
use bevy::prelude::*;

pub(crate) struct LobbyPlugin;

impl Plugin for LobbyPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(LobbyEntrancePlugin)
            .add_systems(Update, dispatch_lobby_actions.in_set(UiActionSet))
            .add_systems(
                Update,
                (animate_lobby_seat_hover, handle_lobby_seat_right_click)
                    .chain()
                    .in_set(ClientUpdateSet::Input),
            );
    }
}
