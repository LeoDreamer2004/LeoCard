use std::error::Error;
use std::fmt;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct MahjongLinkRuleSet {}

impl MahjongLinkRuleSet {
    pub const MIN_PLAYERS: u8 = 1;
    pub const MAX_PLAYERS: u8 = 8;
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct LinkCell {
    pub row: u8,
    pub column: u8,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct LinkPoint {
    pub row: i8,
    pub column: i8,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct LinkMovement {
    pub from: LinkCell,
    pub to: LinkCell,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum PushSide {
    Left,
    Right,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum MahjongLinkAction {
    Match {
        first: LinkCell,
        second: LinkCell,
    },
    Push {
        row: u8,
        from: PushSide,
        distance: u8,
    },
    Pass,
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct MahjongLinkOutcome {
    pub action: MahjongLinkAction,
    pub points: u16,
    pub path: Vec<LinkPoint>,
    pub movements: Vec<LinkMovement>,
    pub complete: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum MahjongLinkError {
    InvalidDeck,
    InvalidPlayer,
    NotPlayersTurn,
    MatchComplete,
    InvalidCell,
    TilesDoNotMatch,
    PathBlocked,
    InvalidPush,
    PushOutOfBounds,
    PushAlreadyUsed,
    InvalidPlayerCount,
}

impl fmt::Display for MahjongLinkError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::InvalidPlayerCount => "麻将连连看支持 1 至 8 人",
            Self::InvalidDeck => "需要万、筒、条及字牌各四张，共 136 张",
            Self::InvalidPlayer => "玩家不在本局中",
            Self::NotPlayersTurn => "还没有轮到你操作",
            Self::MatchComplete => "本局已经结束",
            Self::InvalidCell => "请选择棋盘上的麻将牌",
            Self::TilesDoNotMatch => "只能消除两张相同的牌",
            Self::PathBlocked => "连线被挡住，或需要超过两次转弯",
            Self::InvalidPush => "请选择非空的一排和大于零的推动距离",
            Self::PushOutOfBounds => "推动后的麻将不能超出边界",
            Self::PushAlreadyUsed => "每回合只能推动一次，请消除一对牌或跳过",
        })
    }
}

impl Error for MahjongLinkError {}
