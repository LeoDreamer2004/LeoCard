use crate::app::presentation::{TableSeat, TableSeats};
use bevy::prelude::*;

pub(super) struct BlackjackSeatLayout {
    pub portrait: Node,
    pub hands: Node,
    pub width: f32,
    pub wager_height: f32,
}

impl BlackjackSeatLayout {
    pub(super) const OPPONENT_AREA_WIDTH: f32 = if cfg!(target_os = "android") {
        210.0
    } else {
        236.0
    };
    pub(super) const HAND_SCALE: f32 = if cfg!(target_os = "android") {
        1.5
    } else {
        1.0
    };
    pub(super) const ACTION_SCALE: f32 = if cfg!(target_os = "android") {
        1.4
    } else {
        1.0
    };
    pub(super) const ACTION_BOTTOM: f32 = if cfg!(target_os = "android") {
        218.0
    } else {
        172.0
    };
    pub(super) const DEALER_WIDTH: f32 = if cfg!(target_os = "android") {
        544.0
    } else {
        470.0
    };
    pub(super) const DEALER_CARD_SIZE: Vec2 = if cfg!(target_os = "android") {
        Vec2::new(78.0, 109.2)
    } else {
        Vec2::new(60.0, 84.0)
    };

    pub(super) fn new(relative: u8) -> Self {
        let seat = TableSeats::new(6).seat(relative);
        if cfg!(target_os = "android") {
            return Self::compact(seat);
        }
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

    fn compact(seat: TableSeat) -> Self {
        let own = matches!(seat, TableSeat::Bottom);
        let upper = matches!(seat, TableSeat::Left(1) | TableSeat::Right(1));
        let lower = matches!(seat, TableSeat::Left(0) | TableSeat::Right(0));
        let width = if own {
            486.0
        } else {
            Self::OPPONENT_AREA_WIDTH
        };
        let top = if upper {
            px(94)
        } else if matches!(seat, TableSeat::Top) {
            px(0)
        } else {
            Val::Auto
        };
        Self {
            width,
            wager_height: 32.0,
            portrait: Node {
                position_type: PositionType::Absolute,
                left: px(seat.column(12.0, 766.0, Vec2::new(18.0, 1150.0))),
                top,
                bottom: if own {
                    px(12)
                } else if lower {
                    px(284)
                } else {
                    Val::Auto
                },
                margin: UiRect::top(px(if upper { -4 } else { 0 })),
                width: px(112),
                height: px(if own { 122 } else { 144 }),
                padding: UiRect::top(px(if own { 0 } else { 8 })),
                ..default()
            },
            hands: Node {
                position_type: PositionType::Absolute,
                left: px(seat.column(640.0, 640.0, Vec2::new(250.0, 1030.0)) - width * 0.5),
                top,
                bottom: if own {
                    px(8)
                } else if lower {
                    px(288)
                } else {
                    Val::Auto
                },
                width: px(width),
                height: px(if own { 204 } else { 136 }),
                padding: UiRect::all(px(if own { 0 } else { 6 })),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                column_gap: px(6),
                ..default()
            },
        }
    }
}
