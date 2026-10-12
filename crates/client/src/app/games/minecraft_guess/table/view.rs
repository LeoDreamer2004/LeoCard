use super::super::state::GuessScroll;
use super::super::view::GuessTable;
use super::cells::{CellSkin, TableCell};
use super::rows::GuessEmptyRows;
use super::style::ROW_HEIGHT;
use crate::app::presentation::{Scrollbar, spawn_node};
use bevy::prelude::*;
use bevy::ui::RelativeCursorPosition;
use leocard_minecraft_guess::{Catalogue, GuessRecord, GuessResult, MatchQuality, ValueDirection};

impl GuessTable<'_> {
    pub(in super::super) fn attributes(&self, commands: &mut Commands, parent: Entity) {
        let example = Catalogue::bundled()
            .search(self.game.round.kind, "")
            .next()
            .expect("nonempty catalogue");
        let attribute_width = 85.0 / example.attributes.len() as f32;
        let grid = spawn_node(
            commands,
            parent,
            Node {
                width: percent(100),
                min_height: px(0),
                flex_grow: 1.0,
                column_gap: px(8),
                ..default()
            },
            None,
        );
        let list = spawn_node(
            commands,
            grid,
            Node {
                flex_grow: 1.0,
                flex_basis: px(0),
                min_width: px(0),
                min_height: px(0),
                flex_direction: FlexDirection::Column,
                padding: UiRect {
                    left: px(2),
                    right: px(2),
                    top: px(6),
                    bottom: px(2),
                },
                ..default()
            },
            None,
        );
        commands.entity(list).insert(self.game_assets.inset());
        let header = row(commands, list, 54.0);
        TableCell::spawn(commands, header, 15.0, CellSkin::Heading, self.game_assets).label(
            commands,
            "猜测",
            self.assets,
        );
        for attribute in &example.attributes {
            let label = match attribute.label.as_str() {
                "创造模式物品栏分类" => "创造分类",
                "首次加入版本" => "加入版本",
                other => other,
            };
            TableCell::spawn(
                commands,
                header,
                attribute_width,
                CellSkin::Heading,
                self.game_assets,
            )
            .label(commands, label, self.assets);
        }
        let scroll = spawn_node(
            commands,
            list,
            Node {
                width: percent(100),
                height: px(0),
                min_height: px(0),
                flex_basis: px(0),
                flex_grow: 1.0,
                flex_direction: FlexDirection::Column,
                overflow: Overflow::scroll_y(),
                ..default()
            },
            None,
        );
        commands.entity(scroll).insert((
            GuessScroll,
            RelativeCursorPosition::default(),
            ScrollPosition(Vec2::new(0.0, self.state.scroll_y)),
        ));
        for (index, guess) in self.game.round.guesses.iter().enumerate() {
            self.guess_row(commands, scroll, guess, index, attribute_width);
        }
        GuessEmptyRows::spawn(commands, scroll, self.game.round.guesses.len());
        Scrollbar::spawn(commands, grid, scroll, self.game_assets.scrollbar());
    }

    fn guess_row(
        &self,
        commands: &mut Commands,
        parent: Entity,
        guess: &GuessRecord,
        index: usize,
        width: f32,
    ) {
        let row = row(commands, parent, ROW_HEIGHT);
        let name_quality = if guess.result.is_correct() {
            MatchQuality::Exact
        } else {
            MatchQuality::Wrong
        };
        let player = self
            .game
            .players
            .iter()
            .find(|player| player.id.0 == guess.player)
            .map(|player| player.name.as_str())
            .unwrap_or("已离开玩家");
        let icon = match &guess.result {
            GuessResult::Known { entry, .. } => Some(self.game_assets.icon(entry)),
            GuessResult::Unknown { .. } => None,
        };
        TableCell::spawn(
            commands,
            row,
            15.0,
            CellSkin::Feedback(name_quality),
            self.game_assets,
        )
        .revealing(commands, index, 0)
        .guess(commands, guess.result.name(), player, icon, self.assets);
        let GuessResult::Known { entry, matches, .. } = &guess.result else {
            TableCell::spawn(commands, row, 85.0, CellSkin::Unknown, self.game_assets)
                .revealing(commands, index, 1)
                .label(commands, "此条目不存在", self.assets);
            return;
        };
        for (column, (attribute, feedback)) in entry.attributes.iter().zip(matches).enumerate() {
            let value = attribute.text();
            let label = match feedback.direction {
                Some(ValueDirection::Up) => format!("{value}\u{00a0}↑"),
                Some(ValueDirection::Down) => format!("{value}\u{00a0}↓"),
                None => value,
            };
            TableCell::spawn(
                commands,
                row,
                width,
                CellSkin::Feedback(feedback.quality),
                self.game_assets,
            )
            .revealing(commands, index, column + 1)
            .label(commands, &label, self.assets);
        }
    }
}

fn row(commands: &mut Commands, parent: Entity, minimum: f32) -> Entity {
    spawn_node(
        commands,
        parent,
        Node {
            width: percent(100),
            height: px(minimum),
            flex_shrink: 0.0,
            align_items: AlignItems::Stretch,
            ..default()
        },
        None,
    )
}
