#[derive(Clone, Copy, Default, PartialEq, Eq)]
pub(crate) enum ShopCategory {
    #[default]
    Props,
    AvatarFrames,
}

impl ShopCategory {
    pub const ALL: [Self; 2] = [Self::Props, Self::AvatarFrames];

    pub const fn title(self) -> &'static str {
        match self {
            Self::Props => "道具",
            Self::AvatarFrames => "头像框",
        }
    }
}

#[derive(Default)]
pub(crate) struct ShopUiState {
    pub open: bool,
    pub category: ShopCategory,
}
