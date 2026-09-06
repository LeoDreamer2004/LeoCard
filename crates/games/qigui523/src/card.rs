use std::cmp::Ordering;
use std::fmt;

/// 点数按真实牌面命名；游戏中的大小由 [`QiGuiRank::strength`] 决定。
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum QiGuiRank {
    Four,
    Six,
    Eight,
    Nine,
    Ten,
    Jack,
    Queen,
    King,
    Ace,
    Three,
    Two,
    Five,
    /// 大小王共享同一个点数；大王用黑桃、小王用梅花表示。
    Joker,
    Seven,
}

impl QiGuiRank {
    /// 从小到大的完整点数顺序。
    pub const IN_STRENGTH_ORDER: [Self; 14] = [
        Self::Four,
        Self::Six,
        Self::Eight,
        Self::Nine,
        Self::Ten,
        Self::Jack,
        Self::Queen,
        Self::King,
        Self::Ace,
        Self::Three,
        Self::Two,
        Self::Five,
        Self::Joker,
        Self::Seven,
    ];

    pub const fn strength(self) -> u8 {
        match self {
            Self::Four => 0,
            Self::Six => 1,
            Self::Eight => 2,
            Self::Nine => 3,
            Self::Ten => 4,
            Self::Jack => 5,
            Self::Queen => 6,
            Self::King => 7,
            Self::Ace => 8,
            Self::Three => 9,
            Self::Two => 10,
            Self::Five => 11,
            Self::Joker => 12,
            Self::Seven => 13,
        }
    }

    pub const fn is_joker(self) -> bool {
        matches!(self, Self::Joker)
    }

    pub const fn score(self) -> u16 {
        match self {
            Self::Five => 5,
            Self::Ten | Self::King => 10,
            _ => 0,
        }
    }
}

impl fmt::Display for QiGuiRank {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let text = match self {
            Self::Four => "4",
            Self::Six => "6",
            Self::Eight => "8",
            Self::Nine => "9",
            Self::Ten => "10",
            Self::Jack => "J",
            Self::Queen => "Q",
            Self::King => "K",
            Self::Ace => "A",
            Self::Three => "3",
            Self::Two => "2",
            Self::Five => "5",
            Self::Joker => "王",
            Self::Seven => "7",
        };
        f.write_str(text)
    }
}

/// 花色从方块到黑桃递增。
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum QiGuiSuit {
    Diamond,
    Club,
    Heart,
    Spade,
}

impl QiGuiSuit {
    pub const IN_STRENGTH_ORDER: [Self; 4] = [Self::Diamond, Self::Club, Self::Heart, Self::Spade];

    pub const fn strength(self) -> u8 {
        match self {
            Self::Diamond => 0,
            Self::Club => 1,
            Self::Heart => 2,
            Self::Spade => 3,
        }
    }
}

impl fmt::Display for QiGuiSuit {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Diamond => "♦",
            Self::Club => "♣",
            Self::Heart => "♥",
            Self::Spade => "♠",
        })
    }
}

/// 一张物理牌。`deck` 用来区分多副牌中的重复牌，从 0 开始。
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct QiGuiCard {
    deck: u8,
    rank: QiGuiRank,
    suit: QiGuiSuit,
}

impl QiGuiCard {
    pub const fn suited(deck: u8, suit: QiGuiSuit, rank: QiGuiRank) -> Self {
        assert!(
            !rank.is_joker() || matches!(suit, QiGuiSuit::Club | QiGuiSuit::Spade),
            "joker rank only supports club (small) or spade (big)"
        );
        Self { deck, rank, suit }
    }

    pub const fn deck(self) -> u8 {
        self.deck
    }

    pub const fn rank(self) -> QiGuiRank {
        self.rank
    }

    pub const fn suit(self) -> QiGuiSuit {
        self.suit
    }

    pub const fn score(self) -> u16 {
        self.rank.score()
    }

    /// 不包含 `deck`；同花色同点数的不同副牌具有相同的玩法强度。
    pub(crate) const fn semantic_strength(self) -> (u8, u8) {
        (self.rank.strength(), self.suit.strength())
    }

    pub(crate) const fn suit_points(self) -> u16 {
        self.suit.strength() as u16 + 1
    }

    pub(crate) fn display_cmp(left: &Self, right: &Self) -> Ordering {
        left.semantic_strength()
            .cmp(&right.semantic_strength())
            .then_with(|| left.deck.cmp(&right.deck))
    }
}

impl fmt::Display for QiGuiCard {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match (self.rank, self.suit) {
            (QiGuiRank::Joker, QiGuiSuit::Spade) => f.write_str("大王"),
            (QiGuiRank::Joker, QiGuiSuit::Club) => f.write_str("小王"),
            _ => write!(f, "{}{}", self.suit, self.rank),
        }
    }
}

/// 生成未洗牌的完整牌堆，每副 54 张。
pub fn build_deck(deck_count: u8) -> Vec<QiGuiCard> {
    let mut cards = Vec::with_capacity(usize::from(deck_count) * 54);
    for deck in 0..deck_count {
        for rank in QiGuiRank::IN_STRENGTH_ORDER {
            if rank.is_joker() {
                cards.push(QiGuiCard::suited(deck, QiGuiSuit::Club, rank));
                cards.push(QiGuiCard::suited(deck, QiGuiSuit::Spade, rank));
            } else {
                for suit in QiGuiSuit::IN_STRENGTH_ORDER {
                    cards.push(QiGuiCard::suited(deck, suit, rank));
                }
            }
        }
    }
    cards
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deck_has_54_cards_and_100_points_per_copy() {
        let cards = build_deck(8);
        assert_eq!(cards.len(), 432);
        assert_eq!(cards.iter().map(|card| card.score()).sum::<u16>(), 800);
    }

    #[test]
    fn declared_strength_order_is_exact() {
        for (expected, rank) in QiGuiRank::IN_STRENGTH_ORDER.into_iter().enumerate() {
            assert_eq!(usize::from(rank.strength()), expected);
        }
        assert!(
            QiGuiCard::suited(0, QiGuiSuit::Spade, QiGuiRank::Joker).semantic_strength()
                > QiGuiCard::suited(0, QiGuiSuit::Club, QiGuiRank::Joker).semantic_strength()
        );
        assert!(
            QiGuiCard::suited(0, QiGuiSuit::Spade, QiGuiRank::Four).semantic_strength()
                > QiGuiCard::suited(0, QiGuiSuit::Diamond, QiGuiRank::Four).semantic_strength()
        );
    }
}
