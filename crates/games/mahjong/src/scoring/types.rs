use crate::{MahjongKongKind, MahjongPlayerId, MahjongSuit, MahjongTileKind, MahjongWind, Meld};
use std::fmt;

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Fan {
    BigFourWinds,
    BigThreeDragons,
    AllGreen,
    NineGates,
    FourKongs,
    SevenShiftedPairs,
    ThirteenOrphans,
    AllTerminals,
    LittleFourWinds,
    LittleThreeDragons,
    AllHonors,
    FourConcealedPungs,
    PureTerminalChows,
    QuadrupleChow,
    FourPureShiftedPungs,
    FourPureShiftedChows,
    ThreeKongs,
    AllTerminalsAndHonors,
    SevenPairs,
    GreaterHonorsAndKnittedTiles,
    AllEvenPungs,
    FullFlush,
    PureTripleChow,
    PureShiftedPungs,
    UpperTiles,
    MiddleTiles,
    LowerTiles,
    PureStraight,
    ThreeSuitedTerminalChows,
    PureShiftedChows,
    AllFives,
    TriplePung,
    ThreeConcealedPungs,
    LesserHonorsAndKnittedTiles,
    KnittedStraight,
    UpperFour,
    LowerFour,
    BigThreeWinds,
    MixedStraight,
    ReversibleTiles,
    MixedTripleChow,
    MixedShiftedPungs,
    ChickenHand,
    LastTileDraw,
    LastTileClaim,
    OutWithReplacementTile,
    RobbingTheKong,
    TwoConcealedKongs,
    AllPungs,
    HalfFlush,
    MixedShiftedChows,
    AllTypes,
    MeldedHand,
    TwoDragonPungs,
    OutsideHand,
    FullyConcealedHand,
    TwoMeldedKongs,
    LastTile,
    DragonPung,
    PrevalentWind,
    SeatWind,
    ConcealedHand,
    AllChows,
    TileHog,
    DoublePung,
    TwoConcealedPungs,
    ConcealedKong,
    AllSimples,
    PureDoubleChow,
    MixedDoubleChow,
    ShortStraight,
    TwoTerminalChows,
    PungOfTerminalsOrHonors,
    MeldedKong,
    OneVoidedSuit,
    NoHonors,
    EdgeWait,
    ClosedWait,
    SingleWait,
    SelfDrawn,
    FlowerTiles,
}

impl Fan {
    pub const fn points(self) -> u16 {
        match self {
            Self::BigFourWinds
            | Self::BigThreeDragons
            | Self::AllGreen
            | Self::NineGates
            | Self::FourKongs
            | Self::SevenShiftedPairs
            | Self::ThirteenOrphans => 88,
            Self::AllTerminals
            | Self::LittleFourWinds
            | Self::LittleThreeDragons
            | Self::AllHonors
            | Self::FourConcealedPungs
            | Self::PureTerminalChows => 64,
            Self::QuadrupleChow | Self::FourPureShiftedPungs => 48,
            Self::FourPureShiftedChows | Self::ThreeKongs | Self::AllTerminalsAndHonors => 32,
            Self::SevenPairs
            | Self::GreaterHonorsAndKnittedTiles
            | Self::AllEvenPungs
            | Self::FullFlush
            | Self::PureTripleChow
            | Self::PureShiftedPungs
            | Self::UpperTiles
            | Self::MiddleTiles
            | Self::LowerTiles => 24,
            Self::PureStraight
            | Self::ThreeSuitedTerminalChows
            | Self::PureShiftedChows
            | Self::AllFives
            | Self::TriplePung
            | Self::ThreeConcealedPungs => 16,
            Self::LesserHonorsAndKnittedTiles
            | Self::KnittedStraight
            | Self::UpperFour
            | Self::LowerFour
            | Self::BigThreeWinds => 12,
            Self::MixedStraight
            | Self::ReversibleTiles
            | Self::MixedTripleChow
            | Self::MixedShiftedPungs
            | Self::ChickenHand
            | Self::LastTileDraw
            | Self::LastTileClaim
            | Self::OutWithReplacementTile
            | Self::RobbingTheKong
            | Self::TwoConcealedKongs => 8,
            Self::AllPungs
            | Self::HalfFlush
            | Self::MixedShiftedChows
            | Self::AllTypes
            | Self::MeldedHand
            | Self::TwoDragonPungs => 6,
            Self::OutsideHand
            | Self::FullyConcealedHand
            | Self::TwoMeldedKongs
            | Self::LastTile => 4,
            Self::DragonPung
            | Self::PrevalentWind
            | Self::SeatWind
            | Self::ConcealedHand
            | Self::AllChows
            | Self::TileHog
            | Self::DoublePung
            | Self::TwoConcealedPungs
            | Self::ConcealedKong
            | Self::AllSimples => 2,
            Self::PureDoubleChow
            | Self::MixedDoubleChow
            | Self::ShortStraight
            | Self::TwoTerminalChows
            | Self::PungOfTerminalsOrHonors
            | Self::MeldedKong
            | Self::OneVoidedSuit
            | Self::NoHonors
            | Self::EdgeWait
            | Self::ClosedWait
            | Self::SingleWait
            | Self::SelfDrawn
            | Self::FlowerTiles => 1,
        }
    }

    pub const fn name(self) -> &'static str {
        match self {
            Self::BigFourWinds => "大四喜",
            Self::BigThreeDragons => "大三元",
            Self::AllGreen => "绿一色",
            Self::NineGates => "九莲宝灯",
            Self::FourKongs => "四杠",
            Self::SevenShiftedPairs => "连七对",
            Self::ThirteenOrphans => "十三幺",
            Self::AllTerminals => "清幺九",
            Self::LittleFourWinds => "小四喜",
            Self::LittleThreeDragons => "小三元",
            Self::AllHonors => "字一色",
            Self::FourConcealedPungs => "四暗刻",
            Self::PureTerminalChows => "一色双龙会",
            Self::QuadrupleChow => "一色四同顺",
            Self::FourPureShiftedPungs => "一色四节高",
            Self::FourPureShiftedChows => "一色四步高",
            Self::ThreeKongs => "三杠",
            Self::AllTerminalsAndHonors => "混幺九",
            Self::SevenPairs => "七对",
            Self::GreaterHonorsAndKnittedTiles => "七星不靠",
            Self::AllEvenPungs => "全双刻",
            Self::FullFlush => "清一色",
            Self::PureTripleChow => "一色三同顺",
            Self::PureShiftedPungs => "一色三节高",
            Self::UpperTiles => "全大",
            Self::MiddleTiles => "全中",
            Self::LowerTiles => "全小",
            Self::PureStraight => "清龙",
            Self::ThreeSuitedTerminalChows => "三色双龙会",
            Self::PureShiftedChows => "一色三步高",
            Self::AllFives => "全带五",
            Self::TriplePung => "三同刻",
            Self::ThreeConcealedPungs => "三暗刻",
            Self::LesserHonorsAndKnittedTiles => "全不靠",
            Self::KnittedStraight => "组合龙",
            Self::UpperFour => "大于五",
            Self::LowerFour => "小于五",
            Self::BigThreeWinds => "三风刻",
            Self::MixedStraight => "花龙",
            Self::ReversibleTiles => "推不倒",
            Self::MixedTripleChow => "三色三同顺",
            Self::MixedShiftedPungs => "三色三节高",
            Self::ChickenHand => "无番和",
            Self::LastTileDraw => "妙手回春",
            Self::LastTileClaim => "海底捞月",
            Self::OutWithReplacementTile => "杠上开花",
            Self::RobbingTheKong => "抢杠和",
            Self::TwoConcealedKongs => "双暗杠",
            Self::AllPungs => "碰碰和",
            Self::HalfFlush => "混一色",
            Self::MixedShiftedChows => "三色三步高",
            Self::AllTypes => "五门齐",
            Self::MeldedHand => "全求人",
            Self::TwoDragonPungs => "双箭刻",
            Self::OutsideHand => "全带幺",
            Self::FullyConcealedHand => "不求人",
            Self::TwoMeldedKongs => "双明杠",
            Self::LastTile => "和绝张",
            Self::DragonPung => "箭刻",
            Self::PrevalentWind => "圈风刻",
            Self::SeatWind => "门风刻",
            Self::ConcealedHand => "门前清",
            Self::AllChows => "平和",
            Self::TileHog => "四归一",
            Self::DoublePung => "双同刻",
            Self::TwoConcealedPungs => "双暗刻",
            Self::ConcealedKong => "暗杠",
            Self::AllSimples => "断幺",
            Self::PureDoubleChow => "一般高",
            Self::MixedDoubleChow => "喜相逢",
            Self::ShortStraight => "连六",
            Self::TwoTerminalChows => "老少副",
            Self::PungOfTerminalsOrHonors => "幺九刻",
            Self::MeldedKong => "明杠",
            Self::OneVoidedSuit => "缺一门",
            Self::NoHonors => "无字",
            Self::EdgeWait => "边张",
            Self::ClosedWait => "坎张",
            Self::SingleWait => "单调将",
            Self::SelfDrawn => "自摸",
            Self::FlowerTiles => "花牌",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct FanValue {
    pub fan: Fan,
    pub count: u8,
    pub points: u16,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum WinSource {
    SelfDraw,
    Discard(MahjongPlayerId),
    KongReplacement,
    FlowerReplacement,
    RobbingKong(MahjongPlayerId),
}

impl WinSource {
    pub const fn is_self_draw(self) -> bool {
        matches!(
            self,
            Self::SelfDraw | Self::KongReplacement | Self::FlowerReplacement
        )
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct WinContext {
    pub source: WinSource,
    pub seat_wind: MahjongWind,
    pub prevalent_wind: MahjongWind,
    pub last_wall_tile: bool,
    pub last_of_kind: bool,
    pub flower_count: u8,
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ScoreInput {
    /// 包含和牌张的立牌，不包含副露与花牌。
    pub concealed: Vec<MahjongTileKind>,
    pub melds: Vec<Meld>,
    pub winning_tile: MahjongTileKind,
    pub context: WinContext,
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct MahjongScoreResult {
    pub fans: Vec<FanValue>,
    /// 不含花牌的番数，用于判断是否达到 8 番起和。
    pub points_without_flowers: u16,
    pub flower_points: u8,
    pub total_points: u16,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ScoreError {
    FlowerInHand,
    InvalidTileCount,
    TooManyCopies(MahjongTileKind),
    WinningTileMissing,
    NotComplete,
}

impl fmt::Display for ScoreError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::FlowerInHand => f.write_str("花牌必须先补花，不能留在和牌手牌中"),
            Self::InvalidTileCount => f.write_str("立牌与副露不能组成 14 张标准手牌"),
            Self::TooManyCopies(tile) => write!(f, "牌张数量超过四张：{tile}"),
            Self::WinningTileMissing => f.write_str("立牌中没有指定的和牌张"),
            Self::NotComplete => f.write_str("手牌不是合法和牌结构"),
        }
    }
}

impl std::error::Error for ScoreError {}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum SetKind {
    Chow { suit: MahjongSuit, start: u8 },
    Pung(MahjongTileKind),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct Set {
    pub(super) kind: SetKind,
    pub(super) open: bool,
    pub(super) kong: Option<MahjongKongKind>,
}

#[derive(Clone, Debug)]
pub(super) enum Form {
    Standard {
        sets: Vec<Set>,
        pair: MahjongTileKind,
    },
    SevenPairs {
        shifted: bool,
    },
    ThirteenOrphans,
    Knitted {
        greater: bool,
        straight: bool,
        sets: Vec<Set>,
        pair: Option<MahjongTileKind>,
    },
}
