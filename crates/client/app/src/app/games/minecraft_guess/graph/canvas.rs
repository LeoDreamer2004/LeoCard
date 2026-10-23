use super::super::MinecraftGuessUiAction;
use super::super::controls::{GuessButtonSpec, add_guess_button};
use super::super::view::GuessTable;
use super::art::relation_color;
use super::generation::render_generation_notice;
use super::labels::GraphLabel;
use super::nodes::add_graph_nodes;
use super::state::{GraphCanvas, GraphCurve, GraphDragTarget, GraphEdgeLabel, GraphPane};
use crate::app::presentation::{UiComposer, spawn_node};
use bevy::picking::Pickable;
use bevy::prelude::*;
use bevy::ui::RelativeCursorPosition;

impl GuessTable<'_> {
    pub(super) fn graph_pane(&self, commands: &mut UiComposer, workspace: Entity) {
        let graph = self.game.graph.as_ref().expect("graph mode");
        let (pane, cached) = commands.cached_child(
            workspace,
            "graph-pane",
            (
                self.game.match_id,
                self.game.progress.game_index,
                self.graph_art.revision,
                graph.nodes.len(),
                self.game.round.finished(),
            ),
            Node {
                width: percent(self.state.graph.split * 100.0),
                height: percent(100),
                min_width: px(0),
                flex_shrink: 0.0,
                padding: UiRect::all(px(10)),
                ..default()
            },
        );
        commands
            .entity(pane)
            .insert((GraphPane, self.game_assets.frame()));
        if cached {
            return;
        }
        if graph.nodes.is_empty() {
            render_generation_notice(commands, pane, self.assets);
            return;
        }
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
            Visibility::Hidden,
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
        add_graph_nodes(self, commands, canvas, pane);
        if (cfg!(debug_assertions) || cfg!(feature = "developer")) && !self.game.round.finished() {
            let tools = spawn_node(
                commands,
                pane,
                Node {
                    position_type: PositionType::Absolute,
                    left: px(16),
                    top: px(16),
                    ..default()
                },
                None,
            );
            commands.entity(tools).insert(ZIndex(10));
            add_guess_button(
                commands,
                tools,
                GuessButtonSpec {
                    label: "激活全部",
                    action: Some(MinecraftGuessUiAction::DeveloperActivateAll),
                    width: px(104),
                    height: 34.0,
                    icon: None,
                },
                self.assets,
                self.game_assets,
            );
        }
    }
}
