use super::super::McGuessUiState;
use super::art::{GraphArt, GraphMaterials};
use super::geometry::route_curves;
use super::layout::initial_positions;
use super::lod::GraphLod;
use super::state::{
    GraphCanvas, GraphCurve, GraphEdgeLabel, GraphNodeVisual, GraphPane, GraphWorkspace,
};
use crate::app::runtime::ClientResource;
use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use leocard_protocol::{GameSnapshot, MatchId};

#[derive(Default)]
pub(in super::super) struct GraphProjection {
    canvas: Option<Entity>,
    key: Option<MatchId>,
    revision: u64,
    size: Vec2,
    zoom: f32,
    pan: Vec2,
    positions: Vec<Vec2>,
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
    canvas:
        Query<'w, 's, (Entity, &'static ComputedNode, &'static mut Visibility), With<GraphCanvas>>,
    workspace: Query<'w, 's, &'static ComputedNode, With<GraphWorkspace>>,
    panes: Query<'w, 's, (&'static mut Node, &'static ComputedNode), PaneFilter>,
    nodes: Query<'w, 's, (&'static mut Node, &'static GraphNodeVisual), NodeFilter>,
    curves: Query<'w, 's, (&'static GraphCurve, &'static mut Node), CurveFilter>,
    labels: Query<'w, 's, (&'static GraphEdgeLabel, &'static mut Node), LabelFilter>,
}

pub(in super::super) fn layout_graph(
    mut state: ResMut<McGuessUiState>,
    client: Option<Res<ClientResource>>,
    art: Res<GraphArt>,
    mut materials: GraphMaterials,
    mut layout: GraphLayout,
    mut projection: Local<GraphProjection>,
) {
    let Some(game) = client
        .as_deref()
        .and_then(|c| c.0.model().game_snapshot().and_then(GameSnapshot::mc_guess))
    else {
        return;
    };
    let Some(graph) = &game.graph else { return };
    if graph.nodes.is_empty() {
        return;
    }
    let Ok((canvas_entity, canvas, mut visibility)) = layout.canvas.single_mut() else {
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
    if state.graph.positions.is_empty() {
        state.graph.positions = initial_positions(graph, size);
    }
    if *visibility != Visibility::Inherited {
        *visibility = Visibility::Inherited;
    }
    if projection.canvas == Some(canvas_entity)
        && projection.key == Some(game.match_id)
        && projection.revision == art.revision
        && projection.size == size
        && projection.zoom == state.graph.camera.zoom
        && projection.pan == state.graph.camera.pan
        && projection.positions == state.graph.positions
    {
        return;
    }
    projection.canvas = Some(canvas_entity);
    projection.key = Some(game.match_id);
    projection.revision = art.revision;
    projection.size = size;
    projection.zoom = state.graph.camera.zoom;
    projection.pan = state.graph.camera.pan;
    projection.positions.clone_from(&state.graph.positions);
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
    let curves = route_curves(
        graph,
        &positions,
        GraphLod::at_zoom(state.graph.camera.zoom).node_scale,
    );
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
            let start_control = (geometry.start - bounds.min)
                .extend(geometry.control.x - bounds.min.x)
                .extend(geometry.control.y - bounds.min.y);
            let end_size = Vec4::new(
                geometry.end.x - bounds.min.x,
                geometry.end.y - bounds.min.y,
                extent.x,
                extent.y,
            );
            if material.start_control != start_control || material.end_size != end_size {
                material.start_control = start_control;
                material.end_size = end_size;
            }
        }
    }
    for (index, mut node) in &mut layout.labels {
        let point = curves[index.0].label_position();
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
