use super::super::table::TableHistory;
use super::super::view::GuessTable;
use super::history::HistoryContent;
use super::state::{GraphDragTarget, GraphHistoryPanel, GraphPanelAnimation, GraphWorkspace};
use crate::app::presentation::MUTED;
use bevy::picking::Pickable;
use bevy::prelude::*;
use leocard_ui::{UiComposer, spawn_node};

impl GuessTable<'_> {
    pub(in super::super) fn graph_workspace(&mut self, commands: &mut UiComposer, page: Entity) {
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
        self.graph_pane(commands, workspace);
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
        let right = spawn_node(
            commands,
            workspace,
            Node {
                flex_grow: 1.0,
                flex_basis: px(0),
                min_width: px(0),
                height: percent(100),
                flex_direction: FlexDirection::Column,
                row_gap: px(8),
                ..default()
            },
            None,
        );
        self.graph_history(commands, right);
        self.controls(commands, right);
        self.players(commands, right);
        if self.transition.is_active_for(self.game.match_id)
            && self
                .game
                .graph
                .as_ref()
                .is_some_and(|graph| !graph.nodes.is_empty())
        {
            commands.entity(workspace).insert(Visibility::Hidden);
        }
    }

    fn graph_history(&self, commands: &mut UiComposer, workspace: Entity) {
        let panel = spawn_node(
            commands,
            workspace,
            Node {
                flex_grow: 1.0,
                flex_basis: px(0),
                min_width: px(0),
                height: px(0),
                min_height: px(0),
                ..default()
            },
            None,
        );
        if self
            .game
            .graph
            .as_ref()
            .is_some_and(|graph| graph.nodes.is_empty())
        {
            commands.entity(panel).insert(self.game_assets.frame());
            return;
        }
        let history = &self.state.graph.history;
        if let Some(previous) = &history.previous {
            self.graph_history_content(commands, panel, previous, true);
        }
        self.graph_history_content(
            commands,
            panel,
            history.content.as_ref().expect("synced graph history"),
            false,
        );
    }

    fn graph_history_content(
        &self,
        commands: &mut UiComposer,
        panel: Entity,
        history: &HistoryContent,
        outgoing: bool,
    ) {
        let (content, cached) = commands.cached_child(
            panel,
            format!("wordle-{}", history.node.number),
            (
                self.game.match_id,
                self.game.progress.game_index,
                history.revision,
                outgoing,
            ),
            Node {
                position_type: PositionType::Absolute,
                left: px(0),
                top: px(0),
                width: percent(100),
                height: percent(100),
                min_width: px(0),
                flex_direction: FlexDirection::Column,
                padding: UiRect::all(px(10)),
                row_gap: px(8),
                ..default()
            },
        );
        commands.entity(content).insert((
            self.game_assets.frame(),
            GraphHistoryPanel {
                number: history.node.number,
                revision: history.revision,
                outgoing,
            },
            ZIndex(if outgoing { 0 } else { 1 }),
        ));
        commands
            .entity(content)
            .init(GraphPanelAnimation::default());
        if cached {
            return;
        }
        let selected = &history.node;
        self.graph_heading(commands, content, selected);
        if selected.kind.is_none() {
            self.tag_contents(commands, content, history, outgoing);
        } else {
            self.history_attributes(
                commands,
                content,
                TableHistory {
                    round: &history.round,
                    scroll_y: history.scroll_y,
                    animate: !outgoing,
                },
            );
        }
    }
}
