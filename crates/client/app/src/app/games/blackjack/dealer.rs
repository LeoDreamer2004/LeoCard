use super::BlackjackTable;
use super::cards::{BlackjackCardSpec, add_card};
use super::layout::BlackjackSeatLayout;
use super::playback::BlackjackCardSlot;
use crate::app::presentation::{
    ACCENT, TableStatusEffect, TableStatusSpec, add_playing_card_pile, add_table_play_area,
    add_table_status,
};
use bevy::picking::Pickable;
use bevy::prelude::*;
use leocard_blackjack::HandValue;
use leocard_ui::{UiComposer, spawn_node};

impl BlackjackTable<'_> {
    pub(super) fn dealer(&self, commands: &mut UiComposer, table: Entity) -> Entity {
        let center = add_table_play_area(
            commands,
            table,
            Node {
                position_type: PositionType::Absolute,
                left: percent(50),
                top: px(if cfg!(target_os = "android") {
                    140
                } else {
                    152
                }),
                bottom: px(if cfg!(target_os = "android") {
                    366
                } else {
                    326
                }),
                width: px(BlackjackSeatLayout::DEALER_WIDTH),
                padding: UiRect::all(px(if cfg!(target_os = "android") { 6 } else { 10 })),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                row_gap: px(if cfg!(target_os = "android") { 6 } else { 8 }),
                border_radius: BorderRadius::all(px(12)),
                ..default()
            },
        );
        commands.entity(center).insert((
            UiTransform::from_translation(Val2::px(-BlackjackSeatLayout::DEALER_WIDTH * 0.5, 0.0)),
            Pickable::IGNORE,
        ));
        let title = if self.game.dealer_hidden {
            "庄家 暗牌未揭晓".to_owned()
        } else if self.playback.settlement_started() {
            let total = HandValue::of(&self.game.dealer).total;
            if total > 21 {
                "庄家 爆牌".to_owned()
            } else {
                format!("庄家 {total} 点")
            }
        } else {
            "庄家".to_owned()
        };
        add_table_status(
            commands,
            center,
            TableStatusSpec {
                text: &title,
                font_size: 20.0,
                color: ACCENT,
                effect: TableStatusEffect::Pulse,
                elapsed: None,
            },
            self.assets,
        );
        let row = spawn_node(
            commands,
            center,
            Node {
                width: percent(100),
                height: px(if cfg!(target_os = "android") { 110 } else { 86 }),
                flex_shrink: 0.0,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                column_gap: px(8),
                ..default()
            },
            None,
        );
        let used = self
            .game
            .players
            .iter()
            .flat_map(|view| &view.hands)
            .map(|hand| hand.cards.len())
            .sum::<usize>()
            + self.game.dealer.len()
            + usize::from(self.game.dealer_hidden);
        let pile = add_playing_card_pile(commands, row, 312usize.saturating_sub(used), self.assets);
        let size = BlackjackSeatLayout::DEALER_CARD_SIZE;
        let width = if cfg!(target_os = "android") {
            438.0
        } else {
            358.0
        };
        let cards = spawn_node(
            commands,
            row,
            Node {
                width: px(width),
                height: px(size.y),
                flex_shrink: 0.0,
                position_type: PositionType::Relative,
                ..default()
            },
            None,
        );
        let count = self.game.dealer.len() + usize::from(self.game.dealer_hidden);
        let step = if count <= 5 {
            size.x + 5.0
        } else {
            (width - size.x) / count.saturating_sub(1).max(1) as f32
        };
        let left = (width - size.x - step * count.saturating_sub(1) as f32) * 0.5;
        for index in 0..count {
            add_card(
                commands,
                cards,
                BlackjackCardSpec {
                    slot: BlackjackCardSlot::Dealer(index),
                    face: self.game.dealer.get(index).copied(),
                    node: Node {
                        position_type: PositionType::Absolute,
                        left: px(left + index as f32 * step),
                        width: px(size.x),
                        height: px(size.y),
                        ..default()
                    },
                },
                pile,
                self.playback,
                self.assets,
            );
        }
        pile
    }
}
