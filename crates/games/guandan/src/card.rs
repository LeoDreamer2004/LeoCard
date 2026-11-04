#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum GuandanSuit {
    Spades,
    Hearts,
    Clubs,
    Diamonds,
    Joker,
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct GuandanCard {
    pub deck: u8,
    pub suit: GuandanSuit,
    /// 2..=14 are ordinary ranks; 16 and 17 are the small and big jokers.
    pub rank: u8,
}

impl GuandanCard {
    pub const fn is_joker(self) -> bool {
        matches!(self.suit, GuandanSuit::Joker)
    }

    pub const fn is_wild(self, level: u8) -> bool {
        matches!(self.suit, GuandanSuit::Hearts) && self.rank == level
    }

    pub const fn is_big_joker(self) -> bool {
        self.is_joker() && self.rank == 17
    }
}

pub const fn rank_label(rank: u8) -> &'static str {
    match rank {
        2 => "2",
        3 => "3",
        4 => "4",
        5 => "5",
        6 => "6",
        7 => "7",
        8 => "8",
        9 => "9",
        10 => "10",
        11 => "J",
        12 => "Q",
        13 => "K",
        14 => "A",
        16 => "小王",
        17 => "大王",
        _ => "?",
    }
}

pub fn build_deck() -> Vec<GuandanCard> {
    let mut cards = Vec::with_capacity(108);
    for deck in 0..2 {
        for suit in [
            GuandanSuit::Spades,
            GuandanSuit::Hearts,
            GuandanSuit::Clubs,
            GuandanSuit::Diamonds,
        ] {
            for rank in 2..=14 {
                cards.push(GuandanCard { deck, suit, rank });
            }
        }
        for rank in [16, 17] {
            cards.push(GuandanCard {
                deck,
                suit: GuandanSuit::Joker,
                rank,
            });
        }
    }
    cards
}
