use super::super::view::GuessTable;
use crate::app::presentation::{ACCENT, MUTED, TEXT, UiComposer, add_text, spawn_node};
use bevy::prelude::*;

impl GuessTable<'_> {
    pub(super) fn graph_heading(&self, commands: &mut UiComposer, parent: Entity) {
        let graph = self.game.graph.as_ref().expect("graph mode");
        let selected = graph.selected_node();
        let heading = spawn_node(
            commands,
            parent,
            Node {
                width: percent(100),
                height: px(44),
                min_height: px(44),
                flex_shrink: 0.0,
                padding: UiRect::horizontal(px(38)),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                column_gap: px(10),
                ..default()
            },
            None,
        );
        let number = add_text(
            commands,
            heading,
            graph.selected.to_string(),
            18.0,
            MUTED,
            self.assets,
        );
        commands.entity(number).insert((
            Node {
                position_type: PositionType::Absolute,
                left: px(0),
                top: percent(50),
                ..default()
            },
            UiTransform::from_translation(Val2::new(px(0), percent(-50))),
        ));
        let kind = selected.kind.map_or("标签", |kind| kind.label());
        let value = if selected.revealed() {
            selected.label().to_owned()
        } else if let Some(count) = selected.character_count {
            "_ ".repeat(usize::from(count)).trim_end().to_owned()
        } else {
            "_____".to_owned()
        };
        let label = add_text(
            commands,
            heading,
            format!("{kind}：{value}"),
            23.0,
            if selected.revealed() { ACCENT } else { TEXT },
            self.assets,
        );
        commands.entity(label).insert((
            Node {
                min_width: px(0),
                max_width: percent(100),
                overflow: Overflow::clip(),
                ..default()
            },
            TextLayout::no_wrap(),
        ));
        if let Some(count) = selected.character_count {
            let count = add_text(
                commands,
                heading,
                format!("({count})"),
                17.0,
                MUTED,
                self.assets,
            );
            commands.entity(count).insert((
                Node {
                    flex_shrink: 0.0,
                    ..default()
                },
                TextLayout::no_wrap(),
            ));
        }
    }
}
