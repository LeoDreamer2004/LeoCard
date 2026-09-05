use crate::{PlayerId, Suit, TileKind};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum KongKind {
    Melded,
    Concealed,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum MeldKind {
    Chow,
    Pung,
    Kong(KongKind),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Meld {
    kind: MeldKind,
    tile: TileKind,
    claimed_from: Option<PlayerId>,
}

impl Meld {
    pub const fn chow(suit: Suit, start: u8, claimed_from: PlayerId) -> Self {
        assert!(start >= 1 && start <= 7, "chow must start in 1..=7");
        Self {
            kind: MeldKind::Chow,
            tile: TileKind::suited(suit, start),
            claimed_from: Some(claimed_from),
        }
    }

    pub const fn pung(tile: TileKind, claimed_from: PlayerId) -> Self {
        assert!(!tile.is_flower(), "flower cannot form a pung");
        Self {
            kind: MeldKind::Pung,
            tile,
            claimed_from: Some(claimed_from),
        }
    }

    pub const fn melded_kong(tile: TileKind, claimed_from: PlayerId) -> Self {
        assert!(!tile.is_flower(), "flower cannot form a kong");
        Self {
            kind: MeldKind::Kong(KongKind::Melded),
            tile,
            claimed_from: Some(claimed_from),
        }
    }

    pub const fn concealed_kong(tile: TileKind) -> Self {
        assert!(!tile.is_flower(), "flower cannot form a kong");
        Self {
            kind: MeldKind::Kong(KongKind::Concealed),
            tile,
            claimed_from: None,
        }
    }

    pub const fn kind(self) -> MeldKind {
        self.kind
    }

    pub const fn tile(self) -> TileKind {
        self.tile
    }

    pub const fn claimed_from(self) -> Option<PlayerId> {
        self.claimed_from
    }

    pub const fn is_open(self) -> bool {
        !matches!(self.kind, MeldKind::Kong(KongKind::Concealed))
    }

    pub fn tile_kinds(self) -> Vec<TileKind> {
        match (self.kind, self.tile) {
            (MeldKind::Chow, TileKind::Suited { suit, rank }) => vec![
                TileKind::suited(suit, rank),
                TileKind::suited(suit, rank + 1),
                TileKind::suited(suit, rank + 2),
            ],
            (MeldKind::Pung, tile) => vec![tile; 3],
            (MeldKind::Kong(_), tile) => vec![tile; 4],
            (MeldKind::Chow, _) => unreachable!("Meld::chow always stores a suited tile"),
        }
    }
}
