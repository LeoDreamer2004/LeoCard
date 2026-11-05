use std::fmt;

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum MahjongSuit {
    Characters,
    Bamboo,
    Dots,
}

impl MahjongSuit {
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
pub enum MahjongWind {
    East,
    South,
    West,
    North,
}

impl MahjongWind {
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
pub enum MahjongDragon {
    Red,
    Green,
    White,
}

impl MahjongDragon {
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
pub enum MahjongFlower {
    Spring,
    Summer,
    Autumn,
    Winter,
    Plum,
    Orchid,
    Bamboo,
    Chrysanthemum,
}

impl MahjongFlower {
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
pub enum MahjongTileKind {
    Suited { suit: MahjongSuit, rank: u8 },
    Wind(MahjongWind),
    Dragon(MahjongDragon),
    Flower(MahjongFlower),
}

impl MahjongTileKind {
    pub const fn suited(suit: MahjongSuit, rank: u8) -> Self {
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
            let suit = MahjongSuit::ALL[index / 9];
            return Some(Self::Suited {
                suit,
                rank: (index % 9 + 1) as u8,
            });
        }
        match index {
            27 => Some(Self::Wind(MahjongWind::East)),
            28 => Some(Self::Wind(MahjongWind::South)),
            29 => Some(Self::Wind(MahjongWind::West)),
            30 => Some(Self::Wind(MahjongWind::North)),
            31 => Some(Self::Dragon(MahjongDragon::Red)),
            32 => Some(Self::Dragon(MahjongDragon::Green)),
            33 => Some(Self::Dragon(MahjongDragon::White)),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct MahjongTile {
    kind: MahjongTileKind,
    copy: u8,
}

impl MahjongTile {
    pub const fn new(kind: MahjongTileKind, copy: u8) -> Self {
        assert!(
            (kind.is_flower() && copy == 0) || (!kind.is_flower() && copy < 4),
            "invalid physical mahjong tile copy"
        );
        Self { kind, copy }
    }

    pub const fn kind(self) -> MahjongTileKind {
        self.kind
    }

    pub const fn copy(self) -> u8 {
        self.copy
    }
}

pub fn build_deck() -> Vec<MahjongTile> {
    let mut deck = Vec::with_capacity(144);
    for suit in MahjongSuit::ALL {
        for rank in 1..=9 {
            let kind = MahjongTileKind::suited(suit, rank);
            for copy in 0..4 {
                deck.push(MahjongTile::new(kind, copy));
            }
        }
    }
    for wind in MahjongWind::ALL {
        for copy in 0..4 {
            deck.push(MahjongTile::new(MahjongTileKind::Wind(wind), copy));
        }
    }
    for dragon in MahjongDragon::ALL {
        for copy in 0..4 {
            deck.push(MahjongTile::new(MahjongTileKind::Dragon(dragon), copy));
        }
    }
    for flower in MahjongFlower::ALL {
        deck.push(MahjongTile::new(MahjongTileKind::Flower(flower), 0));
    }
    deck
}

impl fmt::Display for MahjongTileKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Suited { suit, rank } => {
                let suffix = match suit {
                    MahjongSuit::Characters => "万",
                    MahjongSuit::Bamboo => "条",
                    MahjongSuit::Dots => "饼",
                };
                write!(f, "{rank}{suffix}")
            }
            Self::Wind(wind) => f.write_str(match wind {
                MahjongWind::East => "东",
                MahjongWind::South => "南",
                MahjongWind::West => "西",
                MahjongWind::North => "北",
            }),
            Self::Dragon(dragon) => f.write_str(match dragon {
                MahjongDragon::Red => "中",
                MahjongDragon::Green => "发",
                MahjongDragon::White => "白",
            }),
            Self::Flower(flower) => f.write_str(match flower {
                MahjongFlower::Spring => "春",
                MahjongFlower::Summer => "夏",
                MahjongFlower::Autumn => "秋",
                MahjongFlower::Winter => "冬",
                MahjongFlower::Plum => "梅",
                MahjongFlower::Orchid => "兰",
                MahjongFlower::Bamboo => "竹",
                MahjongFlower::Chrysanthemum => "菊",
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
