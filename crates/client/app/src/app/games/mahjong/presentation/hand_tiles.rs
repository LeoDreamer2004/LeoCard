use leocard_mahjong::{MahjongTile, MahjongWinRecord};

/// Separate the winning tile from the concealed hand. On the table, only a self-draw occupies the drawn slot.
pub(crate) struct MahjongDisplayedHand {
    pub regular: Vec<MahjongTile>,
    pub drawn: Option<MahjongTile>,
}

impl MahjongDisplayedHand {
    pub(crate) fn new(tiles: &[MahjongTile], winner: Option<&MahjongWinRecord>) -> Self {
        Self {
            regular: tiles
                .iter()
                .copied()
                .filter(|tile| winner.is_none_or(|winner| winner.winning_tile != *tile))
                .collect(),
            drawn: winner
                .filter(|winner| winner.from.is_none())
                .map(|winner| winner.winning_tile),
        }
    }
}
