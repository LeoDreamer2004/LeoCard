use super::super::view::GuessTable;
use super::labels::GraphLabel;
use super::lod::GraphLod;
use super::state::{GraphDragTarget, GraphNodeVisual};
use super::tooltip::add_node_tooltip;
use crate::app::presentation::{TEXT, UiComposer, spawn_node};
use bevy::picking::Pickable;
use bevy::picking::hover::PickingInteraction;
use bevy::prelude::*;

pub(super) fn add_graph_nodes(
    table: &GuessTable<'_>,
    commands: &mut UiComposer,
    canvas: Entity,
    hints: Entity,
) {
    let graph = table.game.graph.as_ref().expect("graph mode");
    let selected = table.state.viewed_node(table.game).expect("graph node");
    let lod = GraphLod::at_zoom(table.state.graph.camera.zoom);
    for node in &graph.nodes {
        let target = spawn_node(
            commands,
            canvas,
            Node {
                position_type: PositionType::Absolute,
                width: px(64),
                height: px(64),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
            None,
        );
        commands.entity(target).insert((
            GraphDragTarget::Node(node.number),
            PickingInteraction::default(),
            UiTransform {
                translation: Val2::new(percent(-50), px(-32)),
                scale: Vec2::splat(lod.node_scale),
                ..default()
            },
        ));
        if let Some(material) = table.graph_art.nodes.get(usize::from(node.number - 1)) {
            let visual = spawn_node(
                commands,
                target,
                Node {
                    position_type: PositionType::Absolute,
                    left: px(-16),
                    top: px(-16),
                    width: px(96),
                    height: px(96),
                    ..default()
                },
                None,
            );
            commands
                .entity(visual)
                .insert((MaterialNode(material.clone()), Pickable::IGNORE));
        }
        commands.entity(target).init(GraphNodeVisual {
            number: node.number,
            highlight: f32::from(selected == node.number),
            hover: 0.0,
            reveal: f32::from(node.revealed()),
            combo: 0.0,
        });
        if !node.revealed() || node.kind.is_none() {
            let number = table.graph_label(
                commands,
                target,
                GraphLabel::Number(node.number),
                &graph.number_label(node.number),
                if node.kind.is_none() { 19.0 } else { 24.0 },
                TEXT,
            );
            commands.entity(number).insert(ZIndex(1));
        }
        if node.revealed() {
            let label = if node.kind.is_some() {
                format!("{} {}", node.number, node.label())
            } else {
                node.label().to_owned()
            };
            let name = table.graph_label(
                commands,
                target,
                GraphLabel::Name(node.number),
                &label,
                12.0,
                TEXT,
            );
            commands.entity(name).insert((
                Node {
                    position_type: PositionType::Absolute,
                    left: percent(50),
                    top: px(66),
                    ..default()
                },
                lod.name_transform(),
            ));
        }
        add_node_tooltip(table, commands, hints, target, node);
    }
}
