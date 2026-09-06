use crate::{MahjongPlayerId, MahjongSuit, MahjongTileKind};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum MahjongKongKind {
    Melded,
    Concealed,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum MahjongMeldKind {
    Chow,
    Pung,
    Kong(MahjongKongKind),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Meld {
    kind: MahjongMeldKind,
    tile: MahjongTileKind,
    claimed_from: Option<MahjongPlayerId>,
}

impl Meld {
    pub const fn chow(suit: MahjongSuit, start: u8, claimed_from: MahjongPlayerId) -> Self {
        assert!(start >= 1 && start <= 7, "chow must start in 1..=7");
        Self {
            kind: MahjongMeldKind::Chow,
            tile: MahjongTileKind::suited(suit, start),
            claimed_from: Some(claimed_from),
        }
    }

    pub const fn pung(tile: MahjongTileKind, claimed_from: MahjongPlayerId) -> Self {
        assert!(!tile.is_flower(), "flower cannot form a pung");
        Self {
            kind: MahjongMeldKind::Pung,
            tile,
            claimed_from: Some(claimed_from),
        }
    }

    pub const fn melded_kong(tile: MahjongTileKind, claimed_from: MahjongPlayerId) -> Self {
        assert!(!tile.is_flower(), "flower cannot form a kong");
        Self {
            kind: MahjongMeldKind::Kong(MahjongKongKind::Melded),
            tile,
            claimed_from: Some(claimed_from),
        }
    }

    pub const fn concealed_kong(tile: MahjongTileKind) -> Self {
        assert!(!tile.is_flower(), "flower cannot form a kong");
        Self {
            kind: MahjongMeldKind::Kong(MahjongKongKind::Concealed),
            tile,
            claimed_from: None,
        }
    }

    pub const fn kind(self) -> MahjongMeldKind {
        self.kind
    }

    pub const fn tile(self) -> MahjongTileKind {
        self.tile
    }

    pub const fn claimed_from(self) -> Option<MahjongPlayerId> {
        self.claimed_from
    }

    pub const fn is_open(self) -> bool {
        !matches!(self.kind, MahjongMeldKind::Kong(MahjongKongKind::Concealed))
    }

    pub fn tile_kinds(self) -> Vec<MahjongTileKind> {
        match (self.kind, self.tile) {
            (MahjongMeldKind::Chow, MahjongTileKind::Suited { suit, rank }) => vec![
                MahjongTileKind::suited(suit, rank),
                MahjongTileKind::suited(suit, rank + 1),
                MahjongTileKind::suited(suit, rank + 2),
            ],
            (MahjongMeldKind::Pung, tile) => vec![tile; 3],
            (MahjongMeldKind::Kong(_), tile) => vec![tile; 4],
            (MahjongMeldKind::Chow, _) => unreachable!("Meld::chow always stores a suited tile"),
        }
    }
}
