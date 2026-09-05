use std::fmt;

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Suit {
    Characters,
    Bamboo,
    Dots,
}

impl Suit {
    pub const ALL: [Self; 3] = [Self::Characters, Self::Bamboo, Self::Dots];

    pub const fn index(self) -> usize {
        match self {
            Self::Characters => 0,
            Self::Bamboo => 1,
            Self::Dots => 2,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Wind {
    East,
    South,
    West,
    North,
}

impl Wind {
    pub const ALL: [Self; 4] = [Self::East, Self::South, Self::West, Self::North];

    pub const fn index(self) -> usize {
        match self {
            Self::East => 0,
            Self::South => 1,
            Self::West => 2,
            Self::North => 3,
        }
    }

    pub const fn next(self) -> Self {
        match self {
            Self::East => Self::South,
            Self::South => Self::West,
            Self::West => Self::North,
            Self::North => Self::East,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Dragon {
    Red,
    Green,
    White,
}

impl Dragon {
    pub const ALL: [Self; 3] = [Self::Red, Self::Green, Self::White];

    pub const fn index(self) -> usize {
        match self {
            Self::Red => 0,
            Self::Green => 1,
            Self::White => 2,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Flower {
    Spring,
    Summer,
    Autumn,
    Winter,
    Plum,
    Orchid,
    Bamboo,
    Chrysanthemum,
}

impl Flower {
    pub const ALL: [Self; 8] = [
        Self::Spring,
        Self::Summer,
        Self::Autumn,
        Self::Winter,
        Self::Plum,
        Self::Orchid,
        Self::Bamboo,
        Self::Chrysanthemum,
    ];
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum TileKind {
    Suited { suit: Suit, rank: u8 },
    Wind(Wind),
    Dragon(Dragon),
    Flower(Flower),
}

impl TileKind {
    pub const fn suited(suit: Suit, rank: u8) -> Self {
        assert!(rank >= 1 && rank <= 9, "mahjong rank must be in 1..=9");
        Self::Suited { suit, rank }
    }

    pub const fn is_flower(self) -> bool {
        matches!(self, Self::Flower(_))
    }

    pub const fn is_honor(self) -> bool {
        matches!(self, Self::Wind(_) | Self::Dragon(_))
    }

    pub const fn is_terminal(self) -> bool {
        matches!(self, Self::Suited { rank: 1 | 9, .. })
    }

    pub const fn is_terminal_or_honor(self) -> bool {
        self.is_terminal() || self.is_honor()
    }

    pub const fn index34(self) -> Option<usize> {
        match self {
            Self::Suited { suit, rank } => Some(suit.index() * 9 + rank as usize - 1),
            Self::Wind(wind) => Some(27 + wind.index()),
            Self::Dragon(dragon) => Some(31 + dragon.index()),
            Self::Flower(_) => None,
        }
    }

    pub const fn from_index34(index: usize) -> Option<Self> {
        if index < 27 {
            let suit = Suit::ALL[index / 9];
            return Some(Self::Suited {
                suit,
                rank: (index % 9 + 1) as u8,
            });
        }
        match index {
            27 => Some(Self::Wind(Wind::East)),
            28 => Some(Self::Wind(Wind::South)),
            29 => Some(Self::Wind(Wind::West)),
            30 => Some(Self::Wind(Wind::North)),
            31 => Some(Self::Dragon(Dragon::Red)),
            32 => Some(Self::Dragon(Dragon::Green)),
            33 => Some(Self::Dragon(Dragon::White)),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Tile {
    kind: TileKind,
    copy: u8,
}

impl Tile {
    pub const fn new(kind: TileKind, copy: u8) -> Self {
        assert!(
            (kind.is_flower() && copy == 0) || (!kind.is_flower() && copy < 4),
            "invalid physical mahjong tile copy"
        );
        Self { kind, copy }
    }

    pub const fn kind(self) -> TileKind {
        self.kind
    }

    pub const fn copy(self) -> u8 {
        self.copy
    }
}

pub fn build_deck() -> Vec<Tile> {
    let mut deck = Vec::with_capacity(144);
    for suit in Suit::ALL {
        for rank in 1..=9 {
            let kind = TileKind::suited(suit, rank);
            for copy in 0..4 {
                deck.push(Tile::new(kind, copy));
            }
        }
    }
    for wind in Wind::ALL {
        for copy in 0..4 {
            deck.push(Tile::new(TileKind::Wind(wind), copy));
        }
    }
    for dragon in Dragon::ALL {
        for copy in 0..4 {
            deck.push(Tile::new(TileKind::Dragon(dragon), copy));
        }
    }
    for flower in Flower::ALL {
        deck.push(Tile::new(TileKind::Flower(flower), 0));
    }
    deck
}

impl fmt::Display for TileKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Suited { suit, rank } => {
                let suffix = match suit {
                    Suit::Characters => "万",
                    Suit::Bamboo => "条",
                    Suit::Dots => "饼",
                };
                write!(f, "{rank}{suffix}")
            }
            Self::Wind(wind) => f.write_str(match wind {
                Wind::East => "东",
                Wind::South => "南",
                Wind::West => "西",
                Wind::North => "北",
            }),
            Self::Dragon(dragon) => f.write_str(match dragon {
                Dragon::Red => "中",
                Dragon::Green => "发",
                Dragon::White => "白",
            }),
            Self::Flower(flower) => f.write_str(match flower {
                Flower::Spring => "春",
                Flower::Summer => "夏",
                Flower::Autumn => "秋",
                Flower::Winter => "冬",
                Flower::Plum => "梅",
                Flower::Orchid => "兰",
                Flower::Bamboo => "竹",
                Flower::Chrysanthemum => "菊",
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn standard_deck_has_144_distinct_physical_tiles() {
        let deck = build_deck();
        assert_eq!(deck.len(), 144);
        assert_eq!(deck.iter().copied().collect::<HashSet<_>>().len(), 144);
        assert_eq!(
            deck.iter().filter(|tile| tile.kind().is_flower()).count(),
            8
        );
    }
}
