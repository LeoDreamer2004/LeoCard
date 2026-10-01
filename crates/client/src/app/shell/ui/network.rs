//! 网络会话变化驱动的页面、弹窗和开局演出协调。
use super::super::{
    ConnectionEndFeedback, LobbyGameMotion, PageMotion, ProfileMotion, SettingsMotion, UiState,
};
use crate::app::games::GameScreenChange;
use crate::app::presentation::{
    LobbySeatTransitionSnapshot, LobbySeatTransitionSource, StartGameSeatTransition,
};
use crate::app::runtime::{ClientResource, PageErrorState};
use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use leocard_client::{ClientModel, ClientPhaseRef, NetworkState, TcpGameClient};
use leocard_protocol::LobbySnapshot;

pub(crate) struct NetworkUiSnapshot {
    previous_state: NetworkState,
    previous_lobby: Option<LobbySnapshot>,
    was_home: bool,
    was_host: bool,
    lobby_seat_snapshots: Vec<LobbySeatTransitionSnapshot>,
    game_change: GameScreenChange,
}
#[derive(SystemParam)]
pub(crate) struct NetworkUiContext<'w, 's> {
    ui: ResMut<'w, UiState>,
    page_motion: ResMut<'w, PageMotion>,
    lobby_game_motion: ResMut<'w, LobbyGameMotion>,
    settings_motion: ResMut<'w, SettingsMotion>,
    profile_motion: ResMut<'w, ProfileMotion>,
    seat_transition: ResMut<'w, StartGameSeatTransition>,
    lobby_seats: Query<
        'w,
        's,
        (
            &'static LobbySeatTransitionSource,
            &'static ComputedNode,
            &'static UiGlobalTransform,
        ),
    >,
    commands: Commands<'w, 's>,
}
impl NetworkUiContext<'_, '_> {
    pub(crate) fn before_poll(&self, client: &TcpGameClient) -> NetworkUiSnapshot {
        NetworkUiSnapshot {
            previous_state: client.state().clone(),
            previous_lobby: client.model().lobby().cloned(),
            was_home: matches!(
                client.model().phase(),
                ClientPhaseRef::Idle | ClientPhaseRef::Closed
            ),
            was_host: local_player_is_host(client.model()),
            lobby_seat_snapshots: collect_lobby_seats(client.model().lobby(), &self.lobby_seats),
            game_change: GameScreenChange::capture(client.model().game_snapshot()),
        }
    }

    pub(crate) fn after_poll(
        &mut self,
        client: &TcpGameClient,
        before: NetworkUiSnapshot,
        page_error: &mut PageErrorState,
    ) {
        let NetworkUiSnapshot {
            previous_state,
            previous_lobby,
            was_home,
            was_host,
            lobby_seat_snapshots,
            game_change,
        } = before;
        if was_home && client.model().lobby().is_some() {
            self.ui.achievements.open = false;
            self.page_motion.begin();
        }
        if previous_lobby.is_some()
            && client.model().active_game_meta().is_none()
            && (client.model().left_room()
                || client.model().room_closed()
                || matches!(client.state(), NetworkState::Failed(_)))
        {
            self.page_motion.begin();
        }
        if previous_lobby.is_some() && client.model().active_game_meta().is_some() {
            self.page_motion.cancel();
            self.lobby_game_motion.begin();
            self.ui.settings.open = false;
            self.ui.profile.open = false;
            self.ui.profile.player = None;
            self.settings_motion.target_open = false;
            self.settings_motion.progress = 0.0;
            self.profile_motion.target_open = false;
            self.profile_motion.progress = 0.0;
        }
        sync_start_game_transition(
            client.model(),
            previous_lobby.is_some(),
            lobby_seat_snapshots,
            &mut self.seat_transition,
        );

        if let Some(error) = (ConnectionEndFeedback {
            client,
            was_host,
            leaving_room: self.ui.leaving_room,
        })
        .message()
        {
            page_error.error = error;
            self.ui.leaving_room = false;
            self.commands.remove_resource::<ClientResource>();
        }
        if previous_state != *client.state()
            || !game_change.is_transient(client.model().game_snapshot())
        {
            self.ui.dirty = true;
        }
    }
}

fn collect_lobby_seats(
    lobby: Option<&LobbySnapshot>,
    seats: &Query<(
        &LobbySeatTransitionSource,
        &ComputedNode,
        &UiGlobalTransform,
    )>,
) -> Vec<LobbySeatTransitionSnapshot> {
    let Some(lobby) = lobby else {
        return Vec::new();
    };
    seats
        .iter()
        .filter_map(|(source, node, transform)| {
            let player = lobby.players.iter().find(|player| player.id == source.0)?;
            let size = node.size() * node.inverse_scale_factor();
            (size.min_element() > 1.0).then(|| LobbySeatTransitionSnapshot {
                player: player.id,
                center_global: transform.to_scale_angle_translation().2,
                size,
            })
        })
        .collect()
}

fn local_player_is_host(model: &ClientModel) -> bool {
    model.active_game_meta().is_some_and(|game| game.is_host())
        || model
            .lobby()
            .zip(model.you())
            .is_some_and(|(lobby, you)| lobby.host == Some(you))
}

fn sync_start_game_transition(
    model: &ClientModel,
    had_lobby: bool,
    mut seats: Vec<LobbySeatTransitionSnapshot>,
    transition: &mut StartGameSeatTransition,
) {
    let active_game = model.active_game_meta();
    if had_lobby {
        if let Some(game) = active_game {
            seats.retain(|seat| game.players.contains(&seat.player));
            transition.begin(game.match_id, seats);
        } else {
            transition.clear();
        }
    } else if model.lobby().is_some() || active_game.is_none() {
        transition.clear();
    }
}
