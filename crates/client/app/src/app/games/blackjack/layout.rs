use crate::app::presentation::{TableSeat, TableSeats};
use bevy::prelude::*;

pub(super) struct BlackjackSeatLayout {
    pub portrait: Node,
    pub hands: Node,
    pub width: f32,
    pub wager_height: f32,
}

impl BlackjackSeatLayout {
    pub(super) const OPPONENT_AREA_WIDTH: f32 = 236.0;

    pub(super) fn new(relative: u8) -> Self {
        let seat = TableSeats::new(6).seat(relative);
        let own = matches!(seat, TableSeat::Bottom);
        let lower = matches!(seat, TableSeat::Left(0) | TableSeat::Right(0));
        let upper = matches!(seat, TableSeat::Left(1) | TableSeat::Right(1));
        let center = seat.column(640.0, 640.0, Vec2::new(282.0, 998.0));
        let portrait_left = seat.column(12.0, 766.0, Vec2::new(48.0, 1120.0));
        let top = if upper {
            percent(14)
        } else if matches!(seat, TableSeat::Top) {
            px(0)
        } else {
            Val::Auto
        };
        let width = if own {
            590.0
        } else {
            Self::OPPONENT_AREA_WIDTH
        };
        let bottom = if own {
            px(12)
        } else if lower {
            px(174)
        } else {
            Val::Auto
        };
        Self {
            width,
            wager_height: match seat {
                TableSeat::Bottom => 52.0,
                TableSeat::Top => 36.0,
                _ => 56.0,
            },
            portrait: Node {
                position_type: PositionType::Absolute,
                left: px(portrait_left),
                top,
                bottom: if lower { px(194) } else { bottom },
                margin: UiRect::top(px(if upper { 20 } else { 0 })),
                width: px(112),
                height: px(if own { 122 } else { 144 }),
                padding: UiRect::top(px(if own { 0 } else { 8 })),
                ..default()
            },
            hands: Node {
                position_type: PositionType::Absolute,
                left: px(center - width * 0.5),
                top,
                bottom: if own {
                    px(10)
                } else if lower {
                    px(184)
                } else {
                    bottom
                },
                margin: UiRect::top(px(if upper { 10 } else { 0 })),
                width: px(width),
                height: px(match seat {
                    TableSeat::Bottom => 150,
                    TableSeat::Top => 144,
                    _ => 164,
                }),
                padding: UiRect::all(px(if own { 0 } else { 6 })),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                column_gap: px(6),
                border_radius: BorderRadius::all(px(12)),
                ..default()
            },
        }
    }
}
