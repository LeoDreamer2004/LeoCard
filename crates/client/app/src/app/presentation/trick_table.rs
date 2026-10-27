//! 共用的四方牌桌座位和头像旁出牌槽。

use crate::app::shell::SeatSide;
use bevy::prelude::*;
use leocard_ui::{UiComposer, spawn_node};

pub(crate) const TRICK_PORTRAIT_WIDTH: f32 = 96.0 * 1.17;
pub(crate) const TRICK_AVATAR_SIZE: f32 = 52.0 * 1.17;

pub(crate) struct TrickSeatLayout {
    pub side: SeatSide,
    node: Node,
}

impl TrickSeatLayout {
    pub(crate) fn opponent(relative: u8) -> Self {
        let side = match relative {
            1 => SeatSide::Left,
            0 | 2 => SeatSide::Top,
            3 => SeatSide::Right,
            _ => unreachable!("four-sided table seat"),
        };
        let mut node = Node {
            position_type: PositionType::Absolute,
            align_items: AlignItems::Center,
            column_gap: px(if matches!(side, SeatSide::Top) {
                12.0
            } else {
                68.0
            }),
            row_gap: px(7),
            ..default()
        };
        match side {
            SeatSide::Left => {
                node.left = px(42);
                node.top = percent(50);
                node.width = px(TRICK_PORTRAIT_WIDTH + 68.0 + 190.0);
            }
            SeatSide::Top => {
                node.left = percent(32);
                node.right = percent(32);
                if relative == 0 {
                    node.bottom = px(12);
                } else {
                    node.top = px(12);
                }
                node.flex_direction = FlexDirection::Column;
            }
            SeatSide::Right => {
                node.right = px(42);
                node.top = percent(50);
                node.width = px(TRICK_PORTRAIT_WIDTH + 68.0 + 190.0);
                node.justify_content = JustifyContent::FlexEnd;
            }
        }
        Self { side, node }
    }

    pub(crate) fn spawn(self, commands: &mut UiComposer, parent: Entity) -> Entity {
        let slot = spawn_node(commands, parent, self.node, None);
        if !matches!(self.side, SeatSide::Top) {
            commands
                .entity(slot)
                .insert(UiTransform::from_translation(Val2::px(0.0, -38.0)));
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
