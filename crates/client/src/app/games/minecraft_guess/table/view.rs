use super::super::state::GuessScroll;
use super::super::view::GuessTable;
use super::cells::{CellSkin, TableCell};
use super::details::AttributeHint;
use super::rows::GuessEmptyRows;
use super::style::ROW_HEIGHT;
use crate::app::presentation::{Scrollbar, UiComposer, spawn_node};
use bevy::picking::Pickable;
use bevy::prelude::*;
use bevy::ui::RelativeCursorPosition;
use leocard_minecraft_guess::{
    Catalogue, GuessRecord, GuessResult, MatchQuality, RoundView, ValueDirection,
};

pub(in super::super) struct TableHistory<'a> {
    pub round: &'a RoundView,
    pub scroll_y: f32,
    pub animate: bool,
}

impl GuessTable<'_> {
    pub(in super::super) fn attributes(&self, commands: &mut UiComposer, parent: Entity) {
        self.history_attributes(
            commands,
            parent,
            TableHistory {
                round: &self.game.round,
                scroll_y: self.state.scroll_y,
                animate: true,
            },
        );
    }

    pub(in super::super) fn history_attributes(
        &self,
        commands: &mut UiComposer,
        parent: Entity,
        history: TableHistory<'_>,
    ) {
        let example = Catalogue::bundled()
            .search(history.round.kind, "")
            .next()
            .expect("nonempty catalogue");
        let columns = TableColumns::new(example.attributes.len(), self.game.graph.is_some());
        let details = spawn_node(
            commands,
            parent,
            Node {
                position_type: PositionType::Absolute,
                width: percent(100),
                height: percent(100),
                ..default()
            },
            None,
        );
        commands.entity(details).insert(Pickable::IGNORE);
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
        TableCell::spawn(
            commands,
            header,
            columns.name,
            CellSkin::Heading,
            self.game_assets,
            self.game.graph.is_some(),
        )
        .label(commands, "猜测", self.assets);
        for attribute in &example.attributes {
            let label = match attribute.label.as_str() {
                "创造模式物品栏分类" => "创造分类",
                "首次加入版本" => "加入版本",
                other => other,
            };
            TableCell::spawn(
                commands,
                header,
                columns.attribute,
                CellSkin::Heading,
                self.game_assets,
                self.game.graph.is_some(),
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
        if history.animate {
            commands.entity(scroll).insert(GuessScroll);
        }
        commands.entity(scroll).insert((
            RelativeCursorPosition::default(),
            ScrollPosition(Vec2::new(0.0, history.scroll_y)),
        ));
        for (index, guess) in history.round.guesses.iter().enumerate() {
            self.guess_row(
                commands,
                scroll,
                guess,
                history.animate.then_some(index),
                columns,
                details,
            );
        }
        GuessEmptyRows::spawn(commands, scroll, history.round.guesses.len());
        Scrollbar::spawn(commands, grid, scroll, self.game_assets.scrollbar());
    }

    fn guess_row(
        &self,
        commands: &mut UiComposer,
        parent: Entity,
        guess: &GuessRecord,
        index: Option<usize>,
        columns: TableColumns,
        details: Entity,
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
        let name = if columns.name == 100.0 && matches!(guess.result, GuessResult::Unknown { .. }) {
            format!("{}（此条目不存在）", guess.result.name())
        } else {
            guess.result.name().to_owned()
        };
        TableCell::spawn(
            commands,
            row,
            columns.name,
            CellSkin::Feedback(name_quality),
            self.game_assets,
            self.game.graph.is_some(),
        )
        .revealing(commands, index, 0)
        .guess(commands, &name, player, icon, self.assets);
        let GuessResult::Known { entry, matches, .. } = &guess.result else {
            if columns.name < 100.0 {
                TableCell::spawn(
                    commands,
                    row,
                    100.0 - columns.name,
                    CellSkin::Unknown,
                    self.game_assets,
                    self.game.graph.is_some(),
                )
                .revealing(commands, index, 1)
                .label(commands, "此条目不存在", self.assets);
            }
            return;
        };
        for (column, (attribute, feedback)) in entry.attributes.iter().zip(matches).enumerate() {
            let value = AttributeHint::preview(attribute);
            let label = match feedback.direction {
                Some(ValueDirection::Up) => format!("{value}\u{00a0}↑"),
                Some(ValueDirection::Down) => format!("{value}\u{00a0}↓"),
                None => value,
            };
            let cell = TableCell::spawn(
                commands,
                row,
                columns.attribute,
                CellSkin::Feedback(feedback.quality),
                self.game_assets,
                self.game.graph.is_some(),
            )
            .revealing(commands, index, column + 1);
            cell.label(commands, &label, self.assets);
            cell.details(commands, details, attribute, self.assets, self.game_assets);
        }
    }
}

#[derive(Clone, Copy)]
struct TableColumns {
    name: f32,
    attribute: f32,
}

impl TableColumns {
    fn new(attributes: usize, compact: bool) -> Self {
        if attributes == 0 {
            Self {
                name: 100.0,
                attribute: 0.0,
            }
        } else {
            Self {
                name: if compact { 28.0 } else { 15.0 },
                attribute: (if compact { 72.0 } else { 85.0 }) / attributes as f32,
            }
        }
    }
}

fn row(commands: &mut UiComposer, parent: Entity, minimum: f32) -> Entity {
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
