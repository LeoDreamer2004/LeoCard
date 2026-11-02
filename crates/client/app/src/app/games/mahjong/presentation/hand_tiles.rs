use leocard_mahjong::{MahjongTile, MahjongWinRecord};

/// Keep the concealed hand intact; only a self-drawn winning tile occupies the separate right slot.
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
