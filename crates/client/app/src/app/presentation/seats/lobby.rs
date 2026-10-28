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
}
