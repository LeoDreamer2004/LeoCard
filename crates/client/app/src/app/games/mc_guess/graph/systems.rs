use super::super::McGuessUiState;
use super::art::{GraphArt, GraphMaterials};
use super::state::GraphBroadcast;
use crate::app::presentation::StartGameSeatTransition;
use crate::app::runtime::ClientResource;
use crate::app::shell::UiState;
use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use leocard_protocol::GameSnapshot;

#[derive(SystemParam)]
pub(in super::super) struct GraphArtUpdates<'w> {
    art: ResMut<'w, GraphArt>,
    broadcast: ResMut<'w, GraphBroadcast>,
    materials: GraphMaterials<'w>,
}

pub(in super::super) fn sync_graph_state(
    time: Res<Time>,
    client: Option<Res<ClientResource>>,
    mut state: ResMut<McGuessUiState>,
    mut updates: GraphArtUpdates,
    entrance: Res<StartGameSeatTransition>,
    mut ui: ResMut<UiState>,
) {
    let GraphArtUpdates {
        art,
        broadcast,
        materials,
    } = &mut updates;
    let Some((game, graph)) = client
        .as_deref()
        .and_then(|c| c.0.model().game_snapshot().and_then(GameSnapshot::mc_guess))
        .and_then(|game| game.graph.as_ref().map(|graph| (game, graph)))
    else {
        if state.graph.key.is_some() {
            state.graph = super::GraphUiState::default();
        }
        return;
    };
    let key = (game.match_id, game.progress.game_index);
    if state.graph.key != Some(key) {
        state.graph = super::GraphUiState::default();
        state.graph.key = Some(key);
    }
    if graph.nodes.is_empty() {
        return;
    }
    state.graph.selected.get_or_insert(graph.selected);
    if art.key != Some(key) {
        art.begin(key, graph, materials);
    }
    art.sync_sprites(graph, materials);
    let round = state.viewed_round(game);
    let scroll_y = state.scroll_y;
    let local = &mut state.graph;
    if local.seen_guesses != graph.guesses.len() {
        local.seen_guesses = graph.guesses.len();
        if graph.combo.is_some() {
            local.selection_wait = 0.0;
        }
    }
    let target = game
        .round
        .current_player
        .map(|player| (player, graph.selected));
    if broadcast.key != Some(key) || broadcast.target != target {
        broadcast.key = Some(key);
        broadcast.target = target;
        broadcast.started_at = time.elapsed_secs_f64();
    }
    if local.history.sync(
        &graph.nodes[usize::from(local.selected.expect("viewed node") - 1)],
        &round,
        scroll_y,
    ) {
        ui.dirty = true;
    }
    if !entrance.is_active_for(game.match_id) {
        local.intro += time.delta_secs();
    }
    local.selection_wait = (local.selection_wait - time.delta_secs()).max(0.0);
}
