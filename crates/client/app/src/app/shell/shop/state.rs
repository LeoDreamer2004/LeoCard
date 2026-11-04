#[derive(Clone, Copy, Default, PartialEq, Eq)]
pub(crate) enum ShopCategory {
    #[default]
    Items,
}

impl ShopCategory {
    pub const ALL: [Self; 1] = [Self::Items];

    pub const fn title(self) -> &'static str {
        match self {
            Self::Items => "道具",
        }
    }
}

#[derive(Default)]
pub(crate) struct ShopUiState {
    pub open: bool,
    pub category: ShopCategory,
}
