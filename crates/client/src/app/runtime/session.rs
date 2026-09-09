//! 网络快照轮询与客户端状态同步。

use super::{HostRulePreferences, PageErrorState, save_host_rule_preferences};
use crate::app::games::qigui523::only_turn_timer_changed;
use crate::app::games::shengji::only_shengji_transient_progress_changed;
use crate::app::presentation::{
    LobbySeatTransitionSnapshot, LobbySeatTransitionSource, StartGameSeatTransition,
};
use crate::app::shell::UiState;
use bevy::prelude::*;
use leocard_client::{ClientModel, LocalPlayerProfile, NetworkState, TcpGameClient};
use leocard_protocol::LobbySnapshot;

#[derive(Resource)]
pub(crate) struct ClientResource(pub TcpGameClient);

pub(super) fn poll_network(
    mut client: Option<ResMut<ClientResource>>,
    mut host_rules: ResMut<HostRulePreferences>,
    mut page_error: ResMut<PageErrorState>,
    mut profile: ResMut<LocalPlayerProfile>,
    mut ui: ResMut<UiState>,
    mut seat_transition: ResMut<StartGameSeatTransition>,
    lobby_seats: Query<(
        &LobbySeatTransitionSource,
        &ComputedNode,
        &UiGlobalTransform,
    )>,
    mut commands: Commands,
) {
    let Some(client) = client.as_deref_mut() else {
        return;
    };
    let previous_state = client.0.state().clone();
    let previous_game = client.0.model().qigui523_game().cloned();
    let previous_shengji = client.0.model().shengji_game().cloned();
    let previous_lobby = client.0.model().lobby().cloned();
    let lobby_seat_snapshots = collect_lobby_seats(previous_lobby.as_ref(), &lobby_seats);
    let was_host = local_player_is_host(client.0.model());
    if !client.0.poll() {
        return;
    }
    sync_start_game_transition(
        client.0.model(),
        previous_lobby.is_some(),
        lobby_seat_snapshots,
        &mut seat_transition,
    );
    sync_local_profile(client.0.model(), &mut profile, &mut page_error);
    sync_host_rule_preferences(client.0.model(), &mut host_rules, &mut page_error);
    handle_connection_end(&client.0, was_host, &mut page_error, &mut ui, &mut commands);
    if previous_state == *client.0.state()
        && (only_turn_timer_changed(previous_game.as_ref(), client.0.model().qigui523_game())
            || only_shengji_transient_progress_changed(
                previous_shengji.as_ref(),
                client.0.model().shengji_game(),
            ))
    {
        return;
    }
    ui.dirty = true;
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

fn sync_local_profile(
    model: &ClientModel,
    profile: &mut LocalPlayerProfile,
    page_error: &mut PageErrorState,
) {
    let mut changed = model
        .last_finished_match()
        .is_some_and(|(match_id, changes)| profile.apply_finished_match(match_id, changes));
    if let Some(profiles) = model.local_game_profiles() {
        changed |= profile.sync_game_profiles(profiles);
    }
    if changed && let Err(error) = profile.save() {
        page_error.error = Some(error);
    }
}

fn sync_host_rule_preferences(
    model: &ClientModel,
    host_rules: &mut HostRulePreferences,
    page_error: &mut PageErrorState,
) {
    let accepted_rules = model
        .lobby()
        .zip(model.you())
        .filter(|(lobby, you)| lobby.host == Some(*you))
        .map(|(lobby, _)| &lobby.rules);
    if accepted_rules.is_some_and(|rules| host_rules.accept_game_rules(rules))
        && let Err(error) = save_host_rule_preferences(host_rules)
    {
        page_error.error = Some(error);
    }
}

fn handle_connection_end(
    client: &TcpGameClient,
    was_host: bool,
    page_error: &mut PageErrorState,
    ui: &mut UiState,
    commands: &mut Commands,
) {
    let error = if client.model().room_closed() {
        (!was_host && !ui.leaving_room).then(|| "房主结束了游戏".to_owned())
    } else if client.model().left_room() {
        None
    } else if let NetworkState::Failed(error) = client.state() {
        (!ui.leaving_room).then(|| error.clone())
    } else {
        return;
    };
    page_error.error = error;
    ui.leaving_room = false;
    commands.remove_resource::<ClientResource>();
}
