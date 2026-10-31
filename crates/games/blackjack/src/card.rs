#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub enum BlackjackSuit {
    Diamond,
    Club,
    Heart,
    Spade,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
#[repr(u8)]
pub enum BlackjackRank {
    Ace = 1,
    Two,
    Three,
    Four,
    Five,
    Six,
    Seven,
    Eight,
    Nine,
    Ten,
    Jack,
    Queen,
    King,
}

impl BlackjackRank {
    pub const ALL: [Self; 13] = [
        Self::Ace,
        Self::Two,
        Self::Three,
        Self::Four,
        Self::Five,
        Self::Six,
        Self::Seven,
        Self::Eight,
        Self::Nine,
        Self::Ten,
        Self::Jack,
        Self::Queen,
        Self::King,
    ];

    pub const fn points(self) -> u8 {
        match self {
            Self::Ace => 1,
            Self::Jack | Self::Queen | Self::King => 10,
            rank => rank as u8,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub struct BlackjackCard {
    pub suit: BlackjackSuit,
    pub rank: BlackjackRank,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub struct BlackjackHandValue {
    pub total: u16,
    pub soft: bool,
}

impl BlackjackHandValue {
    pub fn of(cards: &[BlackjackCard]) -> Self {
        let mut total = cards.iter().map(|card| u16::from(card.rank.points())).sum();
        let soft = total <= 11 && cards.iter().any(|card| card.rank == BlackjackRank::Ace);
        if soft {
            total += 10;
        }
        Self { total, soft }
    }
}

/// 六副完整牌；`shoe[0]` 是第一张发出的牌，每小局重新洗牌。
pub fn build_shoe() -> Vec<BlackjackCard> {
    (0..6)
        .flat_map(|_| {
            [
                BlackjackSuit::Diamond,
                BlackjackSuit::Club,
                BlackjackSuit::Heart,
                BlackjackSuit::Spade,
            ]
        })
        .flat_map(|suit| BlackjackRank::ALL.map(|rank| BlackjackCard { suit, rank }))
        .collect()
}
