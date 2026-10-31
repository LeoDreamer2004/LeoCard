use bevy::prelude::Vec2;

pub(crate) struct LobbySeatLayout {
    positions: &'static [(f32, f32)],
}

impl LobbySeatLayout {
    pub(crate) fn new(seat_count: u8) -> Self {
        let positions: &[_] = match seat_count {
            3 | 4 => &[(239.0, 290.0), (0.0, 145.0), (239.0, 0.0), (478.0, 145.0)],
            8 => &[
                (239.0, 290.0),
                (82.0, 290.0),
                (0.0, 145.0),
                (82.0, 0.0),
                (239.0, 0.0),
                (396.0, 0.0),
                (478.0, 145.0),
                (396.0, 290.0),
            ],
            6 => &[
                (150.0, 290.0),
                (0.0, 145.0),
                (150.0, 0.0),
                (328.0, 0.0),
                (478.0, 145.0),
                (328.0, 290.0),
            ],
            _ => unreachable!("supported lobby seat counts are three, four, six or eight"),
        };
        Self { positions }
    }

    pub(crate) fn position(&self, seat: u8) -> (f32, f32) {
        self.positions[usize::from(seat)]
    }

    /// Place a seat's action outside the table, including the corner seats of
    /// six/eight-player layouts. Returns an offset in the seat's local space.
    pub(crate) fn outward_popup(&self, seat: u8, popup_size: Vec2) -> Vec2 {
        let min = self
            .positions
            .iter()
            .fold(Vec2::splat(f32::MAX), |min, &(x, y)| {
                min.min(Vec2::new(x, y))
            });
        let max = self
            .positions
            .iter()
            .fold(Vec2::splat(f32::MIN), |max, &(x, y)| {
                max.max(Vec2::new(x, y))
            });
        let half_span = ((max - min) * 0.5).max(Vec2::ONE);
        let (x, y) = self.position(seat);
        let direction = (Vec2::new(x, y) - (min + max) * 0.5) / half_span;
        let seat_size = Vec2::new(142.0, 100.0);
        let centered = (seat_size - popup_size) * 0.5;
        if direction.x.abs() > direction.y.abs() {
            Vec2::new(
                if direction.x < 0.0 {
                    -popup_size.x - 6.0
                } else {
                    seat_size.x + 6.0
                },
                centered.y,
            )
        } else {
            Vec2::new(
                centered.x,
                if direction.y < 0.0 {
                    -popup_size.y - 6.0
                } else {
                    seat_size.y + 6.0
                },
            )
        }
    }
}
