use crate::{MatchId, PlayerPublicInfo, PlayerReferenceChange, SeatId};
use leocard_game_common::PlayerId;
use leocard_shengji::{
    ShengjiBidKind, ShengjiBidTrump, ShengjiBottomFlipReveal, ShengjiCard, ShengjiCategory,
    ShengjiClassifiedPlay, ShengjiFiveTrumpCrossingStage, ShengjiRank, ShengjiRuleSet,
    ShengjiTeamId, ShengjiTrump,
};

use serde::{Deserialize, Serialize};
use std::fmt;

/// 面向单个升级客户端生成的私有快照。
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct ShengjiSnapshot {
    pub match_id: MatchId,
    pub hand_number: u32,
    pub host_port: u16,
    pub you: PlayerId,
    pub host: PlayerId,
    pub rules: ShengjiRuleSet,
    pub players: Vec<ShengjiPlayerView>,
    pub your_hand: Vec<ShengjiCard>,
    /// 仅向接收者公开其本人此前亮过的物理牌；再次亮牌仍以实际手牌为准。
    pub your_exposed_cards: Vec<ShengjiCard>,
    pub levels: [ShengjiRank; 2],
    /// 本局抢亮所使用的级牌。首局无人亮主前庄家尚未公开；首局亮主后随当前
    /// 最高声明实时转移，后续小局则从发牌开始公开上一局已经确定的庄家。
    pub bidding_level: ShengjiRank,
    pub dealer: Option<PlayerId>,
    pub trump: Option<ShengjiTrump>,
    pub declaration: Option<ShengjiDeclarationView>,
    /// 抄底询问中各玩家最近一次的公开选择；首次出牌后清空。
    pub bottom_copy_decisions: Vec<ShengjiBottomCopyDecisionView>,
    pub current_player: Option<PlayerId>,
    pub trick: Option<ShengjiTrickView>,
    /// 本局已经公开打出的实体牌，含当前轮，供记牌及重连恢复使用。
    pub played_cards: Vec<ShengjiCard>,
    /// 甩牌失败的公开演示状态；存在时先展示原甩牌，再收回并显示强制小牌。
    pub throw_failure: Option<ShengjiThrowFailureView>,
    /// 闲家当前总得分，已计入甩牌罚分，但尚未计入未发生的抠底。
    pub collecting_score: u32,
    /// 已埋底的张数；进行中只公开数量，不公开牌面。
    pub buried_count: u8,
    /// 仅最后一位埋底者可见的当前底牌。成功抄底取走旧底后，
    /// 该字段会在新的埋底操作完成前变回空。
    pub your_buried: Vec<ShengjiCard>,
    pub phase: ShengjiPhaseView,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct ShengjiPlayerView {
    pub player: PlayerPublicInfo,
    pub seat: SeatId,
    pub hand_len: u8,
    /// 按首次公开跟牌证明的顺序记录缺门；不包含隐藏手牌信息。
    pub missing_suits: Vec<ShengjiCategory>,
    pub ready: bool,
    pub connected: bool,
    pub auto_play: bool,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct ShengjiDeclarationView {
    pub player: PlayerId,
    pub trump: ShengjiBidTrump,
    pub kind: ShengjiBidKind,
    pub protected: bool,
    /// 亮出的实体牌；同牌面的不同副牌仍具有不同的牌标识。
    pub cards: Vec<ShengjiCard>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct ShengjiBottomCopyDecisionView {
    pub player: PlayerId,
    /// 抄底时公开使用的牌；`None` 表示不抄底。
    pub cards: Option<Vec<ShengjiCard>>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct ShengjiTrickView {
    pub leader: PlayerId,
    pub current_player: PlayerId,
    pub winning_player: PlayerId,
    pub plays: Vec<ShengjiPublicPlay>,
    pub table_points: u16,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct ShengjiPublicPlay {
    pub player: PlayerId,
    /// 甩牌失败时这里只包含被强制打出的最小可失败牌型。
    pub play: ShengjiClassifiedPlay,
    pub throw_penalty: u16,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum ShengjiThrowFailureStage {
    Showing,
    Returning,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct ShengjiThrowFailureView {
    pub player: PlayerId,
    /// 玩家最初尝试甩出的全部实体牌。
    pub attempted: Vec<ShengjiCard>,
    /// 展示结束后实际强制打出的最小可失败牌型。
    pub forced: ShengjiClassifiedPlay,
    pub penalty_points: u16,
    pub stage: ShengjiThrowFailureStage,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum ShengjiPhaseView {
    Dealing {
        cards_remaining: u8,
    },
    BiddingGrace {
        milliseconds_remaining: u16,
        power_outage: bool,
        confirmed_count: u8,
        you_confirmed: bool,
    },
    BottomFlipping {
        reveal: Option<ShengjiBottomFlipReveal>,
    },
    Burying,
    BottomCopying {
        player: PlayerId,
        milliseconds_remaining: u16,
    },
    BottomCopyBurying {
        player: PlayerId,
    },
    FiveTrumpCrossing {
        stage: ShengjiFiveTrumpCrossingStage,
        eligible: Vec<PlayerId>,
        decided: Vec<PlayerId>,
        crossing: Vec<PlayerId>,
        returned: Vec<PlayerId>,
    },
    Playing,
    Finished {
        result: ShengjiHandResultView,
        /// 本局结束后公开，用于核对抠底得分。
        buried: Vec<ShengjiCard>,
    },
    Redealing,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct ShengjiHandResultView {
    /// 每小局独立的结算标识；同一场升级可连续进行多小局。
    pub settlement_id: MatchId,
    pub dealer: PlayerId,
    pub dealer_team: ShengjiTeamId,
    pub collecting_team: ShengjiTeamId,
    pub trick_points: u32,
    pub penalty_adjustment: i32,
    pub kitty_points: u16,
    pub kitty_multiplier: u32,
    pub collecting_score: u32,
    pub promoted_team: ShengjiTeamId,
    pub promoted_steps: u8,
    pub next_dealer: PlayerId,
    pub levels: [ShengjiRank; 2],
    /// 本小局对四名玩家平台积分的实际变动。
    pub reference_changes: Vec<PlayerReferenceChange>,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum ShengjiViolation {
    InvalidPlayer,
    WrongPhase,
    InvalidDeclaration,
    DeclarationCardsNotOwned,
    CounterRequiresPair,
    CounterNotStronger,
    ProtectedSuitCanOnlyBeCounteredByNoTrump,
    DeclarationRequiresJoker,
    NoTrumpCannotOpen,
    NotDealer,
    WrongBuryCount { expected: u8, actual: u8 },
    CrossingNotEligible,
    CrossingAlreadyDecided,
    WrongCrossingCount { expected: u8, actual: u8 },
    CrossingMustIncludeAllTrumps,
    CrossingReturnNotRequired,
    CrossingAlreadyReturned,
    NotBottomCopyPlayer,
    CardsNotOwned,
    NotPlayersTurn,
    MustLeadWithCards,
    ThrowDisabled,
    InvalidPattern,
    WrongCardCount { expected: u8, actual: u8 },
    MustFollowCategory,
    MustFollowStructure,
}

impl fmt::Display for ShengjiViolation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::WrongBuryCount { expected, .. } => {
                return write!(f, "必须埋下 {expected} 张底牌");
            }
            Self::InvalidPlayer => "玩家身份无效",
            Self::WrongPhase => "当前阶段不能执行这个操作",
            Self::InvalidDeclaration => "这些牌不能用于亮主或反主",
            Self::DeclarationCardsNotOwned => "亮出的牌不全在你的手中",
            Self::CounterRequiresPair => "反主必须亮出规则要求的同张牌",
            Self::CounterNotStronger => "只能用更强的主牌反主",
            Self::ProtectedSuitCanOnlyBeCounteredByNoTrump => "自保后只能用无主或更多级牌反主",
            Self::DeclarationRequiresJoker => "带王亮必须同时亮出对应颜色的王",
            Self::NoTrumpCannotOpen => "带王亮时无主只能用于反主",
            Self::NotDealer => "只有庄家可以埋底",
            Self::CardsNotOwned => "选择的牌不全在你的手中",
            Self::CrossingNotEligible => "你本局不符合五主过江条件",
            Self::CrossingAlreadyDecided => "你已经完成过江选择",
            Self::WrongCrossingCount { .. } => "五主过江和归还都必须正好选择五张牌",
            Self::CrossingMustIncludeAllTrumps => "过江牌必须包含你当前的全部主牌",
            Self::CrossingReturnNotRequired => "当前不需要你归还过江牌",
            Self::CrossingAlreadyReturned => "你已经归还过江牌",
            Self::NotBottomCopyPlayer => "当前没有轮到你抄底或重新埋底",
            Self::NotPlayersTurn => "还没有轮到你出牌",
            Self::MustLeadWithCards => "领出时必须出牌",
            Self::ThrowDisabled => "本房间不允许甩牌",
            Self::InvalidPattern => "所选牌不能组成合法牌型",
            Self::WrongCardCount { .. } => "跟牌张数必须与首家相同",
            Self::MustFollowCategory => "手中有该门牌时必须先跟该门",
            Self::MustFollowStructure => "必须优先跟泰坦尼克、拖拉机、三同张或对子结构",
        })
    }
}
