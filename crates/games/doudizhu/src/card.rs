use crate::DoudizhuMode;

/// Unique physical card. Each deck uses 0..52 for 3 through 2, then two jokers.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct DoudizhuCard(pub u8);

impl DoudizhuCard {
    pub const fn rank(self) -> u8 {
        let index = self.0 % 54;
        if index < 52 { index / 4 } else { index - 39 }
    }

    pub const fn suit(self) -> u8 {
        self.0 % 54 % 4
    }

    pub const fn is_red(self) -> bool {
        self.rank() == 14 || (self.rank() < 13 && self.suit().is_multiple_of(2))
    }
}

pub const fn rank_label(rank: u8) -> &'static str {
    match rank {
        0 => "3",
        1 => "4",
        2 => "5",
        3 => "6",
        4 => "7",
        5 => "8",
        6 => "9",
        7 => "10",
        8 => "J",
        9 => "Q",
        10 => "K",
        11 => "A",
        12 => "2",
        13 => "小王",
        14 => "大王",
        _ => "?",
    }
}

pub fn build_deck(mode: DoudizhuMode) -> Vec<DoudizhuCard> {
    (0..mode.deck_count() * 54).map(DoudizhuCard).collect()
}
