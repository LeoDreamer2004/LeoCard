use super::super::assets::EntryIcons;
use super::super::view::GuessTable;
use super::labels::GraphLabel;
use super::state::{GraphDragTarget, GraphNodeVisual};
use crate::app::presentation::{UiComposer, spawn_node};
use bevy::picking::Pickable;
use bevy::prelude::*;

pub(super) fn add_graph_nodes(table: &GuessTable<'_>, commands: &mut UiComposer, canvas: Entity) {
    for node in &table.game.graph.as_ref().expect("graph mode").nodes {
        let target = spawn_node(
            commands,
            canvas,
            Node {
                position_type: PositionType::Absolute,
                width: px(110),
                height: px(94),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                row_gap: px(1),
                ..default()
            },
            None,
        );
        commands.entity(target).insert((
            GraphDragTarget::Node(node.number),
            UiTransform::from_translation(Val2::new(percent(-50), px(-32))),
        ));
        let circle = spawn_node(
            commands,
            target,
            Node {
                width: px(64),
                height: px(64),
                flex_shrink: 0.0,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
            None,
        );
        if let Some(material) = table.graph_art.nodes.get(usize::from(node.number - 1)) {
            commands
                .entity(circle)
                .insert(MaterialNode(material.clone()));
        }
        commands.entity(circle).insert(Pickable::IGNORE);
        commands.entity(target).init(GraphNodeVisual {
            number: node.number,
            highlight: f32::from(
                GraphLabel::Node(node.number)
                    .highlighted(table.game.graph.as_ref().expect("graph mode")),
            ),
            reveal: if node.revealed() { 1.0 } else { 0.0 },
        });
        if let Some(entry) = &node.entry {
            EntryIcons::spawn(commands, circle, table.game_assets.icon(entry), 34.0);
        } else {
            table.graph_label(
                commands,
                circle,
                GraphLabel::Node(node.number),
                &node.number.to_string(),
                18.0,
                Color::srgb(0.5, 0.58, 0.53),
            );
        }
        table.graph_label(
            commands,
            target,
            GraphLabel::Node(node.number),
            node.label(),
            12.0,
            Color::srgb(0.82, 0.87, 0.8),
        );
        if let Some(count) = node.character_count {
            table.graph_label(
                commands,
                target,
                GraphLabel::Node(node.number),
                &format!("{count} 字"),
                10.0,
                Color::srgb(0.6, 0.68, 0.6),
            );
        }
    }
}
