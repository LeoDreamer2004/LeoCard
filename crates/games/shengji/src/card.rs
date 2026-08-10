use std::cmp::Ordering;
use std::fmt;

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Suit {
    Diamond,
    Club,
    Heart,
    Spade,
}

impl Suit {
    pub const ALL: [Self; 4] = [Self::Diamond, Self::Club, Self::Heart, Self::Spade];

    pub const fn bid_strength(self) -> u8 {
        match self {
            Self::Diamond => 0,
            Self::Club => 1,
            Self::Heart => 2,
            Self::Spade => 3,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Rank {
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
    Ace,
    SmallJoker,
    BigJoker,
}

impl Rank {
    pub const LEVELS: [Self; 13] = [
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
        Self::Ace,
    ];

    pub const fn level_index(self) -> Option<u8> {
        match self {
            Self::Two => Some(0),
            Self::Three => Some(1),
            Self::Four => Some(2),
            Self::Five => Some(3),
            Self::Six => Some(4),
            Self::Seven => Some(5),
            Self::Eight => Some(6),
            Self::Nine => Some(7),
            Self::Ten => Some(8),
            Self::Jack => Some(9),
            Self::Queen => Some(10),
            Self::King => Some(11),
            Self::Ace => Some(12),
            Self::SmallJoker | Self::BigJoker => None,
        }
    }

    pub const fn is_level_rank(self) -> bool {
        self.level_index().is_some()
    }

    pub const fn point_value(self) -> u16 {
        match self {
            Self::Five => 5,
            Self::Ten | Self::King => 10,
            _ => 0,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Card {
    deck: u8,
    suit: Option<Suit>,
    rank: Rank,
}

impl Card {
    /// 经典两副牌模式的兼容常量。
    pub const DECK_COUNT: u8 = 2;
    pub const MAX_DECK_COUNT: u8 = 4;

    pub const fn suited(deck: u8, suit: Suit, rank: Rank) -> Self {
        assert!(deck < Self::MAX_DECK_COUNT);
        assert!(rank.is_level_rank());
        Self {
            deck,
            suit: Some(suit),
            rank,
        }
    }

    pub const fn small_joker(deck: u8) -> Self {
        assert!(deck < Self::MAX_DECK_COUNT);
        Self {
            deck,
            suit: None,
            rank: Rank::SmallJoker,
        }
    }

    pub const fn big_joker(deck: u8) -> Self {
        assert!(deck < Self::MAX_DECK_COUNT);
        Self {
            deck,
            suit: None,
            rank: Rank::BigJoker,
        }
    }

    pub const fn deck(self) -> u8 {
        self.deck
    }

    pub const fn suit(self) -> Option<Suit> {
        self.suit
    }

    pub const fn rank(self) -> Rank {
        self.rank
    }

    pub const fn points(self) -> u16 {
        self.rank.point_value()
    }

    pub fn identity_cmp(&self, other: &Self) -> Ordering {
        (self.rank, self.suit, self.deck).cmp(&(other.rank, other.suit, other.deck))
    }

    pub fn same_face(self, other: Self) -> bool {
        self.rank == other.rank && self.suit == other.suit
    }
}

impl fmt::Display for Card {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match (self.suit, self.rank) {
            (None, Rank::SmallJoker) => f.write_str("小王"),
            (None, Rank::BigJoker) => f.write_str("大王"),
            (Some(suit), rank) => write!(f, "{suit:?}{rank:?}"),
            _ => f.write_str("非法牌"),
        }
    }
}

pub fn build_deck() -> Vec<Card> {
    build_deck_for(Card::DECK_COUNT)
}

pub fn build_deck_for(deck_count: u8) -> Vec<Card> {
    assert!((2..=Card::MAX_DECK_COUNT).contains(&deck_count));
    let mut cards = Vec::with_capacity(usize::from(deck_count) * 54);
    for deck in 0..deck_count {
        for suit in Suit::ALL {
            for rank in Rank::LEVELS {
                cards.push(Card::suited(deck, suit, rank));
            }
        }
        cards.push(Card::small_joker(deck));
        cards.push(Card::big_joker(deck));
    }
    cards
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use super::*;

    #[test]
    fn two_decks_have_108_unique_physical_cards_and_200_points() {
        let deck = build_deck();
        assert_eq!(deck.len(), 108);
        assert_eq!(deck.iter().copied().collect::<HashSet<_>>().len(), 108);
        assert_eq!(deck.iter().map(|card| card.points()).sum::<u16>(), 200);
    }

    #[test]
    fn three_decks_have_162_unique_physical_cards_and_300_points() {
        let deck = build_deck_for(3);
        assert_eq!(deck.len(), 162);
        assert_eq!(deck.iter().copied().collect::<HashSet<_>>().len(), 162);
        assert_eq!(deck.iter().map(|card| card.points()).sum::<u16>(), 300);
    }

    #[test]
    fn four_decks_have_216_unique_physical_cards_and_400_points() {
        let deck = build_deck_for(4);
        assert_eq!(deck.len(), 216);
        assert_eq!(deck.iter().copied().collect::<HashSet<_>>().len(), 216);
        assert_eq!(deck.iter().map(|card| card.points()).sum::<u16>(), 400);
    }
}
