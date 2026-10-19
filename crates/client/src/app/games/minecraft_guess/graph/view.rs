use super::super::view::GuessTable;
use super::art::relation_color;
use super::labels::GraphLabel;
use super::nodes::add_graph_nodes;
use super::state::{
    GraphCanvas, GraphCurve, GraphDragTarget, GraphEdgeLabel, GraphHistoryPanel, GraphPane,
    GraphWorkspace,
};
use crate::app::presentation::{MUTED, UiComposer, add_text, spawn_node};
use bevy::picking::Pickable;
use bevy::prelude::*;
use bevy::ui::RelativeCursorPosition;

impl GuessTable<'_> {
    pub(in super::super) fn graph_workspace(&self, commands: &mut UiComposer, page: Entity) {
        let graph = self.game.graph.as_ref().expect("graph mode");
        let workspace = spawn_node(
            commands,
            page,
            Node {
                width: percent(100),
                height: px(0),
                flex_grow: 1.0,
                min_height: px(0),
                ..default()
            },
            None,
        );
        commands.entity(workspace).insert(GraphWorkspace);
        let pane = spawn_node(
            commands,
            workspace,
            Node {
                width: percent(self.state.graph.split * 100.0),
                height: percent(100),
                min_width: px(0),
                flex_shrink: 0.0,
                padding: UiRect::all(px(10)),
                ..default()
            },
            None,
        );
        commands
            .entity(pane)
            .insert((GraphPane, self.game_assets.frame()));
        let canvas = spawn_node(
            commands,
            pane,
            Node {
                width: percent(100),
                height: percent(100),
                min_width: px(0),
                min_height: px(0),
                overflow: Overflow::clip(),
                ..default()
            },
            None,
        );
        commands.entity(canvas).insert((
            GraphCanvas,
            GraphDragTarget::Pan,
            RelativeCursorPosition::default(),
        ));
        for (index, edge) in graph.edges.iter().enumerate() {
            if let Some(material) = self.graph_art.curves.get(index) {
                let curve = spawn_node(
                    commands,
                    canvas,
                    Node {
                        position_type: PositionType::Absolute,
                        ..default()
                    },
                    None,
                );
                commands.entity(curve).insert((
                    GraphCurve(index),
                    MaterialNode(material.clone()),
                    Pickable::IGNORE,
                ));
            }
            let label = self.graph_label(
                commands,
                canvas,
                GraphLabel::Edge(index),
                edge.relation.label(),
                11.0,
                relation_color(edge.relation),
            );
            commands.entity(label).insert((
                GraphEdgeLabel(index),
                Node {
                    position_type: PositionType::Absolute,
                    ..default()
                },
                UiTransform::from_translation(Val2::new(percent(-50), percent(-50))),
            ));
        }
        add_graph_nodes(self, commands, canvas);
        let divider = spawn_node(
            commands,
            workspace,
            Node {
                width: px(16),
                height: percent(100),
                flex_shrink: 0.0,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
            None,
        );
        commands.entity(divider).insert(GraphDragTarget::Divider);
        let grip = spawn_node(
            commands,
            divider,
            Node {
                width: px(4),
                height: px(48),
                border_radius: BorderRadius::all(px(3)),
                ..default()
            },
            Some(MUTED.with_alpha(0.65)),
        );
        commands.entity(grip).insert(Pickable::IGNORE);
        self.graph_history(commands, workspace);
        if self.transition.is_active_for(self.game.match_id) {
            commands.entity(workspace).insert(Visibility::Hidden);
        }
    }

    fn graph_history(&self, commands: &mut UiComposer, workspace: Entity) {
        let graph = self.game.graph.as_ref().expect("graph mode");
        let panel = spawn_node(
            commands,
            workspace,
            Node {
                flex_grow: 1.0,
                flex_basis: px(0),
                min_width: px(0),
                height: percent(100),
                overflow: Overflow::clip(),
                ..default()
            },
            None,
        );
        let content = spawn_node(
            commands,
            panel,
            Node {
                width: percent(100),
                height: percent(100),
                min_width: px(0),
                flex_direction: FlexDirection::Column,
                padding: UiRect::all(px(10)),
                row_gap: px(8),
                ..default()
            },
            None,
        );
        commands
            .entity(content)
            .insert((self.game_assets.frame(), GraphHistoryPanel));
        let selected = graph.selected_node();
        self.graph_heading(commands, content);
        if selected.kind.is_none() {
            add_text(
                commands,
                content,
                "周围节点全部猜中后，标签自动揭晓。",
                14.0,
                MUTED,
                self.assets,
            );
        } else {
            self.attributes(commands, content);
        }
    }
}
