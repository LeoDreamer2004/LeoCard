//! 共用牌桌的玩家框和头像旁出牌槽。

use super::{SeatSide, TableSeat, TableSeats};
use bevy::prelude::*;
use leocard_ui::{UiComposer, spawn_node};

pub(crate) const TRICK_PORTRAIT_WIDTH: f32 = 96.0 * 1.17;
pub(crate) const TRICK_AVATAR_SIZE: f32 = 52.0 * 1.17;

pub(crate) struct TrickSeatLayout {
    pub side: SeatSide,
    node: Node,
    offset_y: f32,
}

impl TrickSeatLayout {
    pub(crate) fn new(relative: u8, count: u8, play_width: f32) -> Self {
        let seats = TableSeats::new(count);
        let seat = seats.seat(relative);
        let side = seat.side();
        let compact = count <= 4;
        let gap = if matches!(side, SeatSide::Top) {
            if compact { 12.0 } else { 8.0 }
        } else if compact {
            68.0
        } else {
            24.0
        };
        let mut node = Node {
            position_type: PositionType::Absolute,
            align_items: AlignItems::Center,
            justify_content: match side {
                SeatSide::Left => JustifyContent::FlexStart,
                SeatSide::Top => JustifyContent::Center,
                SeatSide::Right => JustifyContent::FlexEnd,
            },
            column_gap: px(gap),
            row_gap: px(if compact { 7 } else { 5 }),
            ..default()
        };
        if !compact {
            node.min_height = px(if matches!(side, SeatSide::Top) {
                140
            } else {
                124
            });
        }
        let mut offset_y = 0.0;
        match seat {
            TableSeat::Bottom | TableSeat::Top => {
                node.left = percent(32);
                node.right = percent(32);
                let edge = px(if compact { 12 } else { 4 });
                if matches!(seat, TableSeat::Bottom) {
                    node.bottom = edge;
                } else {
                    node.top = edge;
                }
                if compact {
                    node.flex_direction = FlexDirection::Column;
                }
            }
            TableSeat::Left(row) | TableSeat::Right(row) => {
                let inset = px(if compact { 42 } else { 78 });
                if matches!(seat, TableSeat::Left(_)) {
                    node.left = inset;
                } else {
                    node.right = inset;
                }
                node.width = px(TRICK_PORTRAIT_WIDTH + gap + play_width);
                if compact {
                    node.top = percent(50);
                    offset_y = -38.0;
                } else if row == 0 {
                    node.bottom = px(36);
                } else if row == seats.side_rows() - 1 {
                    node.top = px(78);
                } else {
                    node.top = percent(50);
                    offset_y = -62.0;
                }
            }
        }
        Self {
            side,
            node,
            offset_y,
        }
    }

    pub(crate) fn spawn(self, commands: &mut UiComposer, parent: Entity) -> Entity {
        let slot = spawn_node(commands, parent, self.node, None);
        if self.offset_y != 0.0 {
            commands
                .entity(slot)
                .insert(UiTransform::from_translation(Val2::px(0.0, self.offset_y)));
        }
        slot
    }
}

pub(crate) fn spawn_trick_play_area(
    commands: &mut UiComposer,
    parent: Entity,
    side: SeatSide,
) -> Entity {
    spawn_node(
        commands,
        parent,
        Node {
            width: px(190),
            min_width: px(190),
            min_height: px(112),
            align_items: AlignItems::Center,
            justify_content: match side {
                SeatSide::Left => JustifyContent::FlexStart,
                SeatSide::Top => JustifyContent::Center,
                SeatSide::Right => JustifyContent::FlexEnd,
            },
            ..default()
        },
        None,
    )
}
