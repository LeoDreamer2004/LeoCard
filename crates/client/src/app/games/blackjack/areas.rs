use super::layout::BlackjackSeatLayout;
use crate::app::presentation::{UiComposer, add_table_play_area};
use bevy::prelude::*;
use leocard_protocol::TABLE_SEAT_COUNT;
use std::array;

pub(super) fn add_blackjack_hand_areas(
    commands: &mut UiComposer,
    table: Entity,
) -> [Entity; TABLE_SEAT_COUNT as usize] {
    array::from_fn(|relative| {
        let layout = BlackjackSeatLayout::new(relative as u8);
        let area = commands.keyed_child(
            table,
            format!("blackjack-seat-area:{relative}"),
            (layout.hands,),
        );
        if relative != 0 {
            let background = add_table_play_area(
                commands,
                area,
                Node {
                    position_type: PositionType::Absolute,
                    width: percent(100),
                    height: percent(100),
                    left: px(0),
                    top: px(0),
                    ..default()
                },
            );
            commands.entity(background).insert(ZIndex(-10));
        }
        area
    })
}
