use super::super::view::GuessTable;
use crate::app::presentation::{UiComposer, add_text, spawn_node};
use bevy::picking::Pickable;
use bevy::prelude::*;
use leocard_minecraft_guess::GraphView;

#[derive(Clone, Copy)]
pub(super) enum GraphLabel {
    Node(u16),
    Edge(usize),
}

#[derive(Component)]
pub(in super::super) struct GraphLabelText {
    pub(super) source: GraphLabel,
    pub(super) color: Color,
}

impl GraphLabel {
    pub fn highlighted(self, graph: &GraphView) -> bool {
        match self {
            Self::Node(number) => {
                number == graph.selected || graph.nodes[usize::from(number - 1)].revealed()
            }
            Self::Edge(index) => {
                let edge = &graph.edges[index];
                edge.active || edge.source == graph.selected || edge.target == graph.selected
            }
        }
    }

    pub fn opacity(self, graph: &GraphView) -> f32 {
        if self.highlighted(graph) { 1.0 } else { 0.7 }
    }
}

impl GuessTable<'_> {
    pub(super) fn graph_label(
        &self,
        commands: &mut UiComposer,
        parent: Entity,
        source: GraphLabel,
        text: &str,
        size: f32,
        color: Color,
    ) -> Entity {
        let graph = self.game.graph.as_ref().expect("graph mode");
        let opacity = source.opacity(graph);
        let root = spawn_node(
            commands,
            parent,
            Node {
                flex_shrink: 0.0,
                ..default()
            },
            None,
        );
        commands.entity(root).insert(Pickable::IGNORE);
        // Symmetric glyph passes form a thin outline; no shadow or label backdrop.
        let outline = Color::srgba_u8(36, 38, 49, 200);
        for offset in [
            Vec2::new(-0.75, 0.0),
            Vec2::new(0.75, 0.0),
            Vec2::new(0.0, -0.75),
            Vec2::new(0.0, 0.75),
            Vec2::new(-0.53, -0.53),
            Vec2::new(0.53, -0.53),
            Vec2::new(-0.53, 0.53),
            Vec2::new(0.53, 0.53),
        ] {
            let stroke = add_text(
                commands,
                root,
                text,
                size,
                outline.with_alpha(outline.alpha() * opacity),
                self.assets,
            );
            commands.entity(stroke).insert((
                GraphLabelText {
                    source,
                    color: outline,
                },
                Node {
                    position_type: PositionType::Absolute,
                    left: px(offset.x),
                    top: px(offset.y),
                    width: percent(100),
                    height: percent(100),
                    ..default()
                },
                TextLayout::no_wrap(),
                Pickable::IGNORE,
            ));
        }
        let label = add_text(
            commands,
            root,
            text,
            size,
            color.with_alpha(color.alpha() * opacity),
            self.assets,
        );
        commands.entity(label).insert((
            GraphLabelText { source, color },
            TextLayout::no_wrap(),
            Pickable::IGNORE,
        ));
        root
    }
}
