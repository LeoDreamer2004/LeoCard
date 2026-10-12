use leocard_protocol::GameKind;

pub(super) struct LobbySeatLayout {
    positions: &'static [(f32, f32)],
}

impl LobbySeatLayout {
    pub(super) fn new(game: GameKind) -> Self {
        let positions: &[_] = match game {
            GameKind::Shengji | GameKind::Mahjong => {
                &[(239.0, 290.0), (0.0, 145.0), (239.0, 0.0), (478.0, 145.0)]
            }
            GameKind::MinecraftGuess => &[
                (239.0, 290.0),
                (82.0, 290.0),
                (0.0, 145.0),
                (82.0, 0.0),
                (239.0, 0.0),
                (396.0, 0.0),
                (478.0, 145.0),
                (396.0, 290.0),
            ],
            GameKind::QiGui523 | GameKind::TexasHoldem | GameKind::Uno => &[
                (150.0, 290.0),
                (0.0, 145.0),
                (150.0, 0.0),
                (328.0, 0.0),
                (478.0, 145.0),
                (328.0, 290.0),
            ],
        };
        Self { positions }
    }

    pub(super) fn position(&self, seat: u8) -> (f32, f32) {
        self.positions[usize::from(seat)]
    }
}
