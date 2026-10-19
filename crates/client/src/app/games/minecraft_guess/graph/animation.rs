use super::super::MinecraftGuessUiState;
use super::art::{GraphArt, GraphMaterials, node_color, relation_color};
use super::labels::{GraphLabel, GraphLabelText};
use super::state::{GraphHistoryPanel, GraphNodeVisual};
use crate::app::presentation::{TransitionVisuals, UiComposer, ease_out_cubic, fade_panel};
use crate::app::runtime::ClientResource;
use bevy::prelude::*;

pub(in super::super) fn animate_graph_nodes(
    time: Res<Time>,
    state: Res<MinecraftGuessUiState>,
    client: Option<Res<ClientResource>>,
    art: Res<GraphArt>,
    mut materials: GraphMaterials,
    mut nodes: Query<(&mut GraphNodeVisual, &mut UiTransform)>,
    mut labels: Query<(&GraphLabelText, &mut TextColor)>,
) {
    let Some(graph) = client
        .as_deref()
        .and_then(|c| c.0.model().minecraft_guess_game())
        .and_then(|game| game.graph.as_ref())
    else {
        return;
    };
    let step = (time.delta_secs() * 10.0).min(1.0);
    for (mut visual, mut transform) in &mut nodes {
        let node = &graph.nodes[usize::from(visual.number - 1)];
        let highlighted = GraphLabel::Node(visual.number).highlighted(graph);
        visual.highlight += (f32::from(highlighted) - visual.highlight) * step;
        visual.reveal += (f32::from(node.revealed()) - visual.reveal) * step;
        let entrance = ease_out_cubic(
            ((state.graph.intro - visual.number as f32 * 0.012) / 0.45).clamp(0.0, 1.0),
        );
        transform.scale = Vec2::splat(0.8 + entrance * 0.2);
        if let Some(mut material) = art
            .nodes
            .get(usize::from(visual.number - 1))
            .and_then(|h| materials.nodes.get_mut(h))
        {
            material.color = node_color(node.kind)
                .with_alpha(0.7 + visual.highlight * 0.3)
                .to_linear();
            material.state = Vec4::new(
                visual.highlight,
                f32::from(graph.selected == visual.number),
                visual.reveal,
                f32::from(node.kind.is_none()),
            );
        }
    }
    for (index, edge) in graph.edges.iter().enumerate() {
        let highlight = GraphLabel::Edge(index).highlighted(graph);
        if let Some(mut material) = art
            .curves
            .get(index)
            .and_then(|h| materials.curves.get_mut(h))
        {
            material.color = relation_color(edge.relation)
                .with_alpha((state.graph.intro / 0.45).min(1.0))
                .to_linear();
            material.appearance.x += (f32::from(edge.active) - material.appearance.x) * step;
            material.appearance.y += (f32::from(highlight) - material.appearance.y) * step;
            material.appearance.z = match edge.directed {
                None => 0.0,
                Some(true) => 1.0,
                Some(false) => 2.0,
            };
            material.appearance.w = time.elapsed_secs();
        }
    }
    for (label, mut color) in &mut labels {
        let value = label
            .color
            .with_alpha(label.color.alpha() * label.source.opacity(graph));
        if color.0 != value {
            color.0 = value;
        }
    }
}

pub(in super::super) fn animate_graph_panel(
    mut state: ResMut<MinecraftGuessUiState>,
    panels: Query<Entity, With<GraphHistoryPanel>>,
    children: Query<&Children>,
    mut visuals: TransitionVisuals,
    mut transforms: Query<&mut UiTransform>,
    mut commands: UiComposer,
) {
    if !state.graph.fading {
        return;
    }
    let progress = ease_out_cubic((state.graph.elapsed / 0.28).clamp(0.0, 1.0));
    if progress >= 1.0 {
        state.graph.fading = false;
    }
    for panel in &panels {
        fade_panel(panel, progress, &children, &mut visuals, &mut commands);
        if let Ok(mut transform) = transforms.get_mut(panel) {
            transform.translation.x = px((1.0 - progress) * 42.0);
        }
    }
}
