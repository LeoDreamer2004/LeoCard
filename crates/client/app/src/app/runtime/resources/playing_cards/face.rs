/// 仅描述扑克牌的印刷牌面，不包含任何游戏的点数或强弱规则。
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub(crate) enum CardRank {
    Ace,
    Two,
    Three,
    Four,
    Five,
    Six,
    Seven,
    Eight,
    Nine,
    Ten,
    Jack,
    Queen,
    King,
}

impl CardRank {
    pub(crate) const ALL: [Self; 13] = [
        Self::Ace,
        Self::Two,
        Self::Three,
        Self::Four,
        Self::Five,
        Self::Six,
        Self::Seven,
        Self::Eight,
        Self::Nine,
        Self::Ten,
        Self::Jack,
        Self::Queen,
        Self::King,
    ];

    fn asset_name(self) -> &'static str {
        match self {
            Self::Ace => "A",
            Self::Two => "2",
            Self::Three => "3",
            Self::Four => "4",
            Self::Five => "5",
            Self::Six => "6",
            Self::Seven => "7",
            Self::Eight => "8",
            Self::Nine => "9",
            Self::Ten => "10",
            Self::Jack => "J",
            Self::Queen => "Q",
            Self::King => "K",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub(crate) enum CardSuit {
    Diamond,
    Club,
    Heart,
    Spade,
}

impl CardSuit {
    pub(crate) const ALL: [Self; 4] = [Self::Diamond, Self::Club, Self::Heart, Self::Spade];

    fn asset_name(self) -> &'static str {
        match self {
            Self::Diamond => "Diamonds",
            Self::Club => "Clubs",
            Self::Heart => "Hearts",
            Self::Spade => "Spades",
        }
    }

    pub(super) fn icon_path(self) -> &'static str {
        match self {
            Self::Diamond => "ui/cards/suit-diamond.png",
            Self::Club => "ui/cards/suit-club.png",
            Self::Heart => "ui/cards/suit-heart.png",
            Self::Spade => "ui/cards/suit-spade.png",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub(crate) enum CardFace {
    Suited { rank: CardRank, suit: CardSuit },
    SmallJoker,
    BigJoker,
}

impl CardFace {
    pub(crate) fn all() -> impl Iterator<Item = Self> {
        CardSuit::ALL
            .into_iter()
            .flat_map(|suit| CardRank::ALL.map(|rank| Self::Suited { rank, suit }))
            .chain([Self::SmallJoker, Self::BigJoker])
    }

    pub(crate) fn asset_path(self) -> String {
        let base = "vendor/kenney/boardgame/PNG/Cards";
        match self {
            Self::SmallJoker => format!("{base}/cardJoker.png"),
            Self::BigJoker => format!("{base}/cardJokerBig.png"),
            Self::Suited { rank, suit } => {
                format!("{base}/card{}{}.png", suit.asset_name(), rank.asset_name())
            }
        }
    }
}
