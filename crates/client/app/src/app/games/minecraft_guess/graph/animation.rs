use super::super::MinecraftGuessUiState;
use super::art::{GraphArt, GraphMaterials, node_color, relation_color};
use super::labels::GraphLabel;
use super::lod::GraphLod;
use super::state::{GraphBroadcast, GraphHistoryPanel, GraphNodeVisual, GraphPanelAnimation};
use crate::app::presentation::{TransitionVisuals, UiComposer, UiRoot, ease_out_cubic, fade_panel};
use crate::app::runtime::ClientResource;
use crate::app::shell::UiState;
use bevy::ecs::system::SystemParam;
use bevy::picking::hover::PickingInteraction;
use bevy::prelude::*;

#[derive(SystemParam)]
pub(in super::super) struct GraphPlayback<'w> {
    time: Res<'w, Time>,
    art: Res<'w, GraphArt>,
    broadcast: Res<'w, GraphBroadcast>,
}

pub(in super::super) fn animate_graph_nodes(
    playback: GraphPlayback,
    state: Res<MinecraftGuessUiState>,
    client: Option<Res<ClientResource>>,
    mut materials: GraphMaterials,
    mut nodes: Query<(&mut GraphNodeVisual, &mut UiTransform, &PickingInteraction)>,
) {
    let GraphPlayback {
        time,
        art,
        broadcast,
    } = &playback;
    let Some(game) = client
        .as_deref()
        .and_then(|c| c.0.model().minecraft_guess_game())
    else {
        return;
    };
    let Some(graph) = &game.graph else { return };
    if graph.nodes.is_empty() {
        return;
    }
    let step = 1.0 - (-time.delta_secs().min(1.0 / 30.0) * 12.0).exp();
    let selected = state.graph.selected.unwrap_or(graph.selected);
    let lod = GraphLod::at_zoom(state.graph.camera.zoom);
    for (mut visual, mut transform, interaction) in &mut nodes {
        let node = &graph.nodes[usize::from(visual.number - 1)];
        let highlight = approach(visual.highlight, selected == visual.number, step);
        let hover = approach(visual.hover, *interaction != PickingInteraction::None, step);
        let reveal = approach(visual.reveal, node.revealed(), step);
        let combo = approach(
            visual.combo,
            graph.combo.is_some() && game.can_guess_node(visual.number),
            step,
        );
        if (visual.highlight, visual.hover, visual.reveal, visual.combo)
            != (highlight, hover, reveal, combo)
        {
            visual.highlight = highlight;
            visual.hover = hover;
            visual.reveal = reveal;
            visual.combo = combo;
        }
        let entrance = ease_out_cubic(
            ((state.graph.intro - visual.number as f32 * 0.012) / 0.45).clamp(0.0, 1.0),
        );
        let active = visual.highlight.max(visual.hover);
        let enlargement = if node.kind.is_some() {
            active * 0.16
        } else {
            0.0
        };
        let scale = Vec2::splat((0.8 + entrance * 0.2) * (1.0 + enlargement) * lod.node_scale);
        if transform.scale != scale {
            transform.scale = scale;
        }
        if let Some(mut material) = art
            .nodes
            .get(usize::from(visual.number - 1))
            .and_then(|h| materials.nodes.get_mut(h))
        {
            let color = node_color(node.kind)
                .with_alpha(0.7 + active * 0.3)
                .to_linear();
            let state_vector = Vec4::new(
                if node.kind.is_none() {
                    visual.highlight.max(visual.reveal)
                } else {
                    active.max(visual.reveal)
                },
                visual.highlight,
                visual.reveal,
                f32::from(node.kind.is_none()),
            );
            let acting = broadcast
                .target
                .is_some_and(|(_, number)| number == visual.number);
            let activity = Vec4::new(
                f32::from(acting),
                if acting {
                    (time.elapsed_secs_f64() - broadcast.started_at) as f32
                } else {
                    0.0
                },
                combo,
                if combo > 0.0 {
                    (time.elapsed_secs_f64() % 2.4) as f32
                } else {
                    0.0
                },
            );
            if material.color != color
                || material.state != state_vector
                || material.activity != activity
            {
                material.color = color;
                material.state = state_vector;
                material.activity = activity;
            }
        }
    }
    for (index, edge) in graph.edges.iter().enumerate() {
        let highlight = GraphLabel::Edge(index).highlighted(graph, selected);
        if let Some(mut material) = art
            .curves
            .get(index)
            .and_then(|h| materials.curves.get_mut(h))
        {
            let color = relation_color(edge.relation)
                .with_alpha((state.graph.intro / 0.45).min(1.0))
                .to_linear();
            let activated = approach(material.appearance.x, edge.active, step);
            let emphasis = approach(material.appearance.y, highlight, step);
            let directions = match edge.directed {
                None => 0.0,
                Some(true) => 1.0,
                Some(false) => 2.0,
            };
            let appearance = Vec4::new(activated, emphasis, directions, 0.0);
            if material.color != color || material.appearance != appearance {
                material.color = color;
                material.appearance = appearance;
            }
        }
    }
}

fn approach(value: f32, target: bool, step: f32) -> f32 {
    let target = f32::from(target);
    let next = value + (target - value) * step;
    if (next - target).abs() < 0.001 {
        target
    } else {
        next
    }
}

#[derive(SystemParam)]
pub(in super::super) struct GraphPanelPlayback<'w> {
    time: Res<'w, Time>,
    state: ResMut<'w, MinecraftGuessUiState>,
    ui: ResMut<'w, UiState>,
}

pub(in super::super) fn animate_graph_panel(
    mut playback: GraphPanelPlayback,
    roots: Query<&ComputedNode, With<UiRoot>>,
    mut panels: Query<(
        Entity,
        &ComputedNode,
        &GraphHistoryPanel,
        &mut GraphPanelAnimation,
    )>,
    children: Query<&Children>,
    mut transforms: Query<&mut UiTransform>,
    mut visuals: TransitionVisuals,
    mut commands: UiComposer,
) {
    let Ok(viewport) = roots.single() else {
        return;
    };
    let extent = viewport.size() * viewport.inverse_scale_factor() + Vec2::splat(16.0);
    let GraphPanelPlayback { time, state, ui } = &mut playback;
    let ready = state.graph.history.content.as_ref().is_some_and(|content| {
        panels.iter().any(|(_, computed, panel, _)| {
            !panel.outgoing
                && panel.number == content.node.number
                && panel.revision == content.revision
                && computed.size().min_element() > 0.0
        })
    });
    if state.graph.history.advance(time.delta_secs(), ready) {
        ui.dirty = true;
    }
    for (panel, _, history, mut animation) in &mut panels {
        let pose = state.graph.history.pose(history.outgoing);
        let applied = (history.revision, pose, extent);
        if animation.applied == Some(applied) {
            continue;
        }
        animation.applied = Some(applied);
        if let Ok(mut transform) = transforms.get_mut(panel) {
            let offset = pose.offset * extent;
            transform.translation = Val2::px(offset.x, offset.y);
        }
        fade_panel(panel, pose.opacity, &children, &mut visuals, &mut commands);
    }
}
