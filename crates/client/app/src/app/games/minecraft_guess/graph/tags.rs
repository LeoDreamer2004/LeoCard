use super::super::assets::EntryIcons;
use super::super::state::GuessScroll;
use super::super::view::GuessTable;
use super::history::HistoryContent;
use crate::app::presentation::{MUTED, TEXT, add_text};
use bevy::picking::Pickable;
use bevy::prelude::*;
use bevy::ui::RelativeCursorPosition;
use leocard_minecraft_guess::Entry;
use leocard_ui::{Scrollbar, UiComposer, spawn_node};

impl GuessTable<'_> {
    pub(super) fn tag_contents(
        &self,
        commands: &mut UiComposer,
        parent: Entity,
        history: &HistoryContent,
        outgoing: bool,
    ) {
        let unlocked = history.node.revealed();
        let body = spawn_node(
            commands,
            parent,
            Node {
                width: percent(100),
                height: px(0),
                min_height: px(0),
                flex_grow: 1.0,
                column_gap: px(8),
                ..default()
            },
            None,
        );
        let panel = spawn_node(
            commands,
            body,
            Node {
                flex_grow: 1.0,
                flex_basis: px(0),
                min_width: px(0),
                height: percent(100),
                min_height: px(0),
                padding: UiRect {
                    left: px(2),
                    right: px(2),
                    top: px(6),
                    bottom: px(2),
                },
                flex_direction: FlexDirection::Column,
                ..default()
            },
            None,
        );
        commands.entity(panel).insert(self.game_assets.inset());
        if !unlocked {
            commands.entity(panel).insert(Node {
                width: percent(100),
                height: percent(100),
                min_height: px(0),
                padding: UiRect::all(px(24)),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            });
            let hint = add_text(
                commands,
                panel,
                "周围节点全部猜中后，标签自动揭晓。",
                22.0,
                MUTED,
                self.assets,
            );
            commands.entity(hint).insert((
                Node {
                    max_width: percent(100),
                    ..default()
                },
                TextLayout::justify(Justify::Center),
                Pickable::IGNORE,
            ));
            return;
        }
        let scroll = spawn_node(
            commands,
            panel,
            Node {
                width: percent(100),
                height: px(0),
                min_height: px(0),
                flex_grow: 1.0,
                flex_basis: px(0),
                flex_direction: FlexDirection::Column,
                padding: UiRect::all(px(12)),
                row_gap: px(14),
                overflow: Overflow::scroll_y(),
                ..default()
            },
            None,
        );
        commands.entity(scroll).insert((
            RelativeCursorPosition::default(),
            ScrollPosition(Vec2::new(0.0, history.scroll_y)),
        ));
        if !outgoing {
            commands.entity(scroll).insert(GuessScroll);
        }
        let members = history.node.tag_members();
        for group in members.chunk_by(|a, b| a.kind == b.kind) {
            let heading = add_text(
                commands,
                scroll,
                format!("包含此标签的{}有{}个：", group[0].kind.label(), group.len()),
                19.0,
                TEXT,
                self.assets,
            );
            commands.entity(heading).insert((
                Node {
                    flex_shrink: 0.0,
                    ..default()
                },
                Pickable::IGNORE,
            ));
            for entries in group.chunks(3) {
                let row = spawn_node(
                    commands,
                    scroll,
                    Node {
                        width: percent(100),
                        flex_shrink: 0.0,
                        align_items: AlignItems::Stretch,
                        ..default()
                    },
                    None,
                );
                for entry in entries {
                    self.tag_member(commands, row, entry);
                }
            }
        }
        Scrollbar::spawn(commands, body, scroll, self.game_assets.scrollbar());
    }

    fn tag_member(&self, commands: &mut UiComposer, row: Entity, entry: &Entry) {
        let cell = spawn_node(
            commands,
            row,
            Node {
                width: percent(100.0 / 3.0),
                min_width: px(0),
                min_height: px(56),
                flex_shrink: 0.0,
                padding: UiRect::all(px(6)),
                column_gap: px(8),
                align_items: AlignItems::Center,
                ..default()
            },
            None,
        );
        EntryIcons::spawn(commands, cell, self.game_assets.icon(entry), 36.0);
        let name = add_text(commands, cell, &entry.name, 17.0, TEXT, self.assets);
        commands.entity(name).insert((
            Node {
                flex_grow: 1.0,
                flex_basis: px(0),
                min_width: px(0),
                ..default()
            },
            Pickable::IGNORE,
        ));
    }
}
