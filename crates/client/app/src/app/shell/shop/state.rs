use leocard_protocol::GameKind;

pub(crate) struct ShopUiState {
    pub open: bool,
    pub category: GameKind,
}

impl Default for ShopUiState {
    fn default() -> Self {
        Self {
            open: false,
            category: GameKind::TexasHoldem,
        }
    }
}
