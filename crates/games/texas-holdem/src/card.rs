use std::fmt;

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum TexasHoldemRank {
    Two = 2,
    Three = 3,
    Four = 4,
    Five = 5,
    Six = 6,
    Seven = 7,
    Eight = 8,
    Nine = 9,
    Ten = 10,
    Jack = 11,
    Queen = 12,
    King = 13,
    Ace = 14,
}

impl TexasHoldemRank {
    pub const ALL: [Self; 13] = [
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

    pub const SHORT_DECK: [Self; 9] = [
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

    pub const fn value(self) -> u8 {
        self as u8
    }
}

impl fmt::Display for TexasHoldemRank {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Two => "2",
            Self::Three => "3",
            Self::Four => "4",
            Self::Five => "5",
            Self::Six => "6",
            Self::Seven => "7",
            Self::Eight => "8",
            Self::Nine => "9",
            Self::Ten => "10",
            Self::Jack => "J",
            Self::Queen => "Q",
            Self::King => "K",
            Self::Ace => "A",
        })
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum TexasHoldemSuit {
    Diamond,
    Club,
    Heart,
    Spade,
}

impl TexasHoldemSuit {
    pub const ALL: [Self; 4] = [Self::Diamond, Self::Club, Self::Heart, Self::Spade];
}

impl fmt::Display for TexasHoldemSuit {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Diamond => "♦",
            Self::Club => "♣",
            Self::Heart => "♥",
            Self::Spade => "♠",
        })
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct TexasHoldemCard {
    rank: TexasHoldemRank,
    suit: TexasHoldemSuit,
}

impl TexasHoldemCard {
    pub const fn new(suit: TexasHoldemSuit, rank: TexasHoldemRank) -> Self {
        Self { rank, suit }
    }

    pub const fn rank(self) -> TexasHoldemRank {
        self.rank
    }

    pub const fn suit(self) -> TexasHoldemSuit {
        self.suit
    }
}

impl fmt::Display for TexasHoldemCard {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}{}", self.suit, self.rank)
    }
}

/// 生成未洗牌的 52 张普通牌，或移除 2、3、4、5 后的 36 张短牌牌堆。
pub fn build_deck(short_deck: bool) -> Vec<TexasHoldemCard> {
    let ranks = if short_deck {
        TexasHoldemRank::SHORT_DECK.as_slice()
    } else {
        TexasHoldemRank::ALL.as_slice()
    };
    ranks
        .iter()
        .flat_map(|rank| TexasHoldemSuit::ALL.map(|suit| TexasHoldemCard::new(suit, *rank)))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn standard_and_short_decks_have_the_declared_unique_cards() {
        let standard = build_deck(false);
        let short = build_deck(true);
        assert_eq!(standard.len(), 52);
        assert_eq!(short.len(), 36);
        assert_eq!(standard.iter().copied().collect::<HashSet<_>>().len(), 52);
        assert_eq!(short.iter().copied().collect::<HashSet<_>>().len(), 36);
        assert!(short.iter().all(|card| card.rank().value() >= 6));
    }
}
