use super::super::MinecraftGuessUiState;
use super::art::{GraphArt, GraphMaterials};
use super::geometry::CurveGeometry;
use super::layout::initial_positions;
use super::state::{
    GraphCanvas, GraphCurve, GraphEdgeLabel, GraphNodeVisual, GraphPane, GraphWorkspace,
};
use crate::app::presentation::StartGameSeatTransition;
use crate::app::runtime::ClientResource;
use bevy::ecs::system::SystemParam;
use bevy::prelude::*;

pub(in super::super) fn sync_graph_state(
    time: Res<Time>,
    client: Option<Res<ClientResource>>,
    mut state: ResMut<MinecraftGuessUiState>,
    mut art: ResMut<GraphArt>,
    mut materials: GraphMaterials,
    entrance: Res<StartGameSeatTransition>,
) {
    let Some(game) = client
        .as_deref()
        .and_then(|c| c.0.model().minecraft_guess_game())
    else {
        return;
    };
    let Some(graph) = &game.graph else { return };
    let local = &mut state.graph;
    if local.key != Some(game.match_id) {
        *local = super::GraphUiState::default();
        local.key = Some(game.match_id);
        local.positions = initial_positions(graph);
        local.selected = Some(graph.selected);
    }
    if art.key != Some(game.match_id) {
        art.begin(game.match_id, graph, &mut materials);
    }
    if local.selected != Some(graph.selected) {
        local.selected = Some(graph.selected);
        local.elapsed = 0.0;
        local.fading = true;
        local.selection_wait = 0.6;
    }
    local.elapsed += time.delta_secs();
    if !entrance.is_active_for(game.match_id) {
        local.intro += time.delta_secs();
    }
    local.selection_wait = (local.selection_wait - time.delta_secs()).max(0.0);
}

type NodeFilter = (
    With<GraphNodeVisual>,
    Without<GraphPane>,
    Without<GraphCurve>,
    Without<GraphEdgeLabel>,
);
type PaneFilter = (
    With<GraphPane>,
    Without<GraphCurve>,
    Without<GraphEdgeLabel>,
);
type CurveFilter = (
    Without<GraphNodeVisual>,
    Without<GraphPane>,
    Without<GraphEdgeLabel>,
);
type LabelFilter = (
    Without<GraphNodeVisual>,
    Without<GraphPane>,
    Without<GraphCurve>,
);

#[derive(SystemParam)]
pub(in super::super) struct GraphLayout<'w, 's> {
    canvas: Query<'w, 's, &'static ComputedNode, With<GraphCanvas>>,
    workspace: Query<'w, 's, &'static ComputedNode, With<GraphWorkspace>>,
    panes: Query<'w, 's, (&'static mut Node, &'static ComputedNode), PaneFilter>,
    nodes: Query<'w, 's, (&'static mut Node, &'static GraphNodeVisual), NodeFilter>,
    curves: Query<'w, 's, (&'static GraphCurve, &'static mut Node), CurveFilter>,
    labels: Query<'w, 's, (&'static GraphEdgeLabel, &'static mut Node), LabelFilter>,
}

pub(in super::super) fn layout_graph(
    state: Res<MinecraftGuessUiState>,
    client: Option<Res<ClientResource>>,
    art: Res<GraphArt>,
    mut materials: GraphMaterials,
    mut layout: GraphLayout,
) {
    let Some(game) = client
        .as_deref()
        .and_then(|c| c.0.model().minecraft_guess_game())
    else {
        return;
    };
    let Some(graph) = &game.graph else { return };
    let Ok(canvas) = layout.canvas.single() else {
        return;
    };
    let mut size = canvas.size() * canvas.inverse_scale_factor();
    for (mut pane, computed) in &mut layout.panes {
        if let Ok(workspace) = layout.workspace.single() {
            // Preserve the canvas inset when predicting this frame's divider resize.
            size.x += workspace.size().x * workspace.inverse_scale_factor() * state.graph.split
                - computed.size().x * computed.inverse_scale_factor();
        }
        let width = percent(state.graph.split * 100.0);
        if pane.width != width {
            pane.width = width;
        }
    }
    if size.min_element() <= 0.0 {
        return;
    }
    let positions: Vec<_> = state
        .graph
        .positions
        .iter()
        .map(|p| state.graph.camera.project(*p, size))
        .collect();
    for (mut node, visual) in &mut layout.nodes {
        if let Some(point) = positions.get(usize::from(visual.number - 1)) {
            set_origin(&mut node, *point);
        }
    }
    let curves: Vec<_> = graph
        .edges
        .iter()
        .map(|edge| {
            CurveGeometry::between(
                positions[usize::from(edge.source - 1)],
                positions[usize::from(edge.target - 1)],
                if edge.source < edge.target { 1.0 } else { -1.0 },
            )
        })
        .collect();
    for (index, mut node) in &mut layout.curves {
        let geometry = &curves[index.0];
        let bounds = geometry.bounds();
        let extent = bounds.size();
        set_origin(&mut node, bounds.min);
        if node.width != px(extent.x) {
            node.width = px(extent.x);
        }
        if node.height != px(extent.y) {
            node.height = px(extent.y);
        }
        if let Some(mut material) = art
            .curves
            .get(index.0)
            .and_then(|handle| materials.curves.get_mut(handle))
        {
            material.start_control = (geometry.start - bounds.min)
                .extend(geometry.control.x - bounds.min.x)
                .extend(geometry.control.y - bounds.min.y);
            material.end_size = Vec4::new(
                geometry.end.x - bounds.min.x,
                geometry.end.y - bounds.min.y,
                extent.x,
                extent.y,
            );
        }
    }
    let mut placed = Vec::new();
    for (index, mut node) in &mut layout.labels {
        let point = curves[index.0].label_position(&positions, &placed);
        placed.push(point);
        set_origin(&mut node, point);
    }
}

fn set_origin(node: &mut Node, point: Vec2) {
    if node.left != px(point.x) {
        node.left = px(point.x);
    }
    if node.top != px(point.y) {
        node.top = px(point.y);
    }
}
