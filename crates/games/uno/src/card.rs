use std::cmp::Ordering;
use std::fmt;

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Color {
    Red,
    Yellow,
    Green,
    Blue,
}

impl Color {
    pub const ALL: [Self; 4] = [Self::Red, Self::Yellow, Self::Green, Self::Blue];
}

impl fmt::Display for Color {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Red => "红",
            Self::Yellow => "黄",
            Self::Green => "绿",
            Self::Blue => "蓝",
        })
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Face {
    Number(u8),
    DrawTwo,
    Reverse,
    Skip,
    Wild,
    WildDrawFour,
}

impl Face {
    pub const fn score(self) -> u16 {
        match self {
            Self::Number(value) => value as u16,
            Self::DrawTwo | Self::Reverse | Self::Skip => 20,
            Self::Wild | Self::WildDrawFour => 50,
        }
    }

    pub const fn is_wild(self) -> bool {
        matches!(self, Self::Wild | Self::WildDrawFour)
    }
}

impl fmt::Display for Face {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Number(value) => value.fmt(f),
            Self::DrawTwo => f.write_str("+2"),
            Self::Reverse => f.write_str("反转"),
            Self::Skip => f.write_str("跳过"),
            Self::Wild => f.write_str("万能"),
            Self::WildDrawFour => f.write_str("万能+4"),
        }
    }
}

/// 一张物理牌。`copy` 区分同牌面的不同实体牌，从 0 开始。
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Card {
    color: Option<Color>,
    face: Face,
    copy: u8,
}

impl Card {
    pub const fn number(color: Color, value: u8, copy: u8) -> Self {
        assert!(value <= 9, "UNO number cards are 0..=9");
        assert!(copy < if value == 0 { 1 } else { 2 });
        Self {
            color: Some(color),
            face: Face::Number(value),
            copy,
        }
    }

    pub const fn action(color: Color, face: Face, copy: u8) -> Self {
        assert!(matches!(face, Face::DrawTwo | Face::Reverse | Face::Skip));
        assert!(copy < 2);
        Self {
            color: Some(color),
            face,
            copy,
        }
    }

    pub const fn wild(face: Face, copy: u8) -> Self {
        assert!(matches!(face, Face::Wild | Face::WildDrawFour));
        assert!(copy < 4);
        Self {
            color: None,
            face,
            copy,
        }
    }

    pub const fn color(self) -> Option<Color> {
        self.color
    }

    pub const fn face(self) -> Face {
        self.face
    }

    pub const fn copy(self) -> u8 {
        self.copy
    }

    pub const fn score(self) -> u16 {
        self.face.score()
    }

    pub(crate) fn display_cmp(left: &Self, right: &Self) -> Ordering {
        (left.color, left.face, left.copy).cmp(&(right.color, right.face, right.copy))
    }
}

impl fmt::Display for Card {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.color {
            Some(color) => write!(f, "{color}{}", self.face),
            None => self.face.fmt(f),
        }
    }
}

/// 生成经典 UNO 的 108 张牌。
pub fn build_deck() -> Vec<Card> {
    let mut cards = Vec::with_capacity(108);
    for color in Color::ALL {
        cards.push(Card::number(color, 0, 0));
        for value in 1..=9 {
            cards.push(Card::number(color, value, 0));
            cards.push(Card::number(color, value, 1));
        }
        for face in [Face::DrawTwo, Face::Reverse, Face::Skip] {
            cards.push(Card::action(color, face, 0));
            cards.push(Card::action(color, face, 1));
        }
    }
    for copy in 0..4 {
        cards.push(Card::wild(Face::Wild, copy));
        cards.push(Card::wild(Face::WildDrawFour, copy));
    }
    cards
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classic_deck_has_108_unique_physical_cards() {
        let deck = build_deck();
        assert_eq!(deck.len(), 108);
        assert_eq!(
            deck.iter()
                .copied()
                .collect::<std::collections::HashSet<_>>()
                .len(),
            108
        );
        assert_eq!(
            deck.iter().filter(|card| card.face == Face::Wild).count(),
            4
        );
        assert_eq!(
            deck.iter()
                .filter(|card| card.face == Face::WildDrawFour)
                .count(),
            4
        );
    }
}
