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
        let own = relative == 0;
        let (center, top, portrait_left) = match relative {
            0 => (640.0, Val::Auto, 12.0),
            1 => (282.0, Val::Auto, 48.0),
            2 => (282.0, percent(14), 48.0),
            3 => (640.0, px(0), 766.0),
            4 => (998.0, percent(14), 1120.0),
            5 => (998.0, Val::Auto, 1120.0),
            _ => unreachable!("six-seat blackjack table"),
        };
        let width = if own {
            590.0
        } else {
            Self::OPPONENT_AREA_WIDTH
        };
        let bottom = if own {
            px(12)
        } else if matches!(relative, 1 | 5) {
            px(174)
        } else {
            Val::Auto
        };
        Self {
            width,
            wager_height: match relative {
                0 => 52.0,
                3 => 36.0,
                _ => 56.0,
            },
            portrait: Node {
                position_type: PositionType::Absolute,
                left: px(portrait_left),
                top,
                bottom: if matches!(relative, 1 | 5) {
                    px(194)
                } else {
                    bottom
                },
                margin: UiRect::top(px(if matches!(relative, 2 | 4) { 20 } else { 0 })),
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
                } else if matches!(relative, 1 | 5) {
                    px(184)
                } else {
                    bottom
                },
                margin: UiRect::top(px(if matches!(relative, 2 | 4) { 10 } else { 0 })),
                width: px(width),
                height: px(match relative {
                    0 => 150,
                    3 => 144,
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
