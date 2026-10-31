use crate::{MatchId, PlayerPublicInfo, PlayerReferenceChange, SeatId};
use leocard_game_common::PlayerId;
use leocard_uno::{UnoCard, UnoColor, UnoDirection, UnoFlipSide, UnoPendingDrawKind, UnoRuleSet};
use serde::{Deserialize, Serialize};
use std::fmt;

/// 面向单个 UNO 客户端的私有快照。
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct UnoSnapshot {
    pub match_id: MatchId,
    pub host_port: u16,
    pub you: PlayerId,
    pub host: PlayerId,
    pub rules: UnoRuleSet,
    pub players: Vec<UnoPlayerView>,
    pub your_hand: Vec<UnoCard>,
    pub draw_pile_len: u16,
    /// FLIP 模式下摸牌堆顶部至多六张牌朝下的一面；其他模式为空。
    pub draw_pile_inactive_cards: Vec<UnoCard>,
    pub discard_top: UnoCard,
    /// 从旧到新排列的弃牌堆末尾，用于客户端绘制有轻微错位的牌堆。
    pub discard_pile: Vec<UnoCard>,
    pub current_color: Option<UnoColor>,
    pub flip_side: Option<UnoFlipSide>,
    pub current_player: Option<PlayerId>,
    pub direction: UnoDirection,
    pub pending_draw: u16,
    pub pending_kind: Option<UnoPendingDrawKind>,
    pub challenge_offender: Option<PlayerId>,
    pub pending_skip: u16,
    pub pending_swap: Option<UnoPendingSwapView>,
    /// 牌堆耗尽且双方均无法响应时，当前玩家可任选一张出牌。
    pub sudden_death_free_play: bool,
    /// 只有接收者本人摸到可出的牌并仍在本回合时才为 `Some`。
    pub your_drawn_card: Option<UnoCard>,
    /// 抢出窗口内，只有实际持有匹配牌的非下家会收到这张私有候选牌。
    pub your_jump_in_card: Option<UnoCard>,
    pub uno_exposed: Vec<PlayerId>,
    pub uno_declared: Vec<PlayerId>,
    pub can_call_uno: bool,
    pub phase: UnoPhaseView,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum UnoPendingSwapView {
    SwapOneTarget { player: PlayerId },
    SwapOneGive { player: PlayerId, target: PlayerId },
    ForceTrade { player: PlayerId },
    ChooseColor { player: PlayerId },
    SevenSwap { player: PlayerId },
    ColorRoulette { player: PlayerId },
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct UnoPlayerView {
    pub player: PlayerPublicInfo,
    pub seat: SeatId,
    pub hand_len: u8,
    /// FLIP 模式公开的手牌背面，按当前手牌顺序紧密排列。
    pub inactive_hand: Vec<UnoCard>,
    pub ready: bool,
    pub connected: bool,
    pub auto_play: bool,
    pub skipped_turns: u16,
    pub eliminated: bool,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum UnoPhaseView {
    Playing,
    Finished {
        winner: PlayerId,
        results: Vec<UnoPlayerResult>,
        remaining_hands: Vec<UnoRevealedHand>,
        reference_changes: Vec<PlayerReferenceChange>,
    },
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct UnoPlayerResult {
    pub player: PlayerId,
    pub hand_score: u16,
    pub placement: u8,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct UnoRevealedHand {
    pub player: PlayerId,
    pub cards: Vec<UnoCard>,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum UnoViolation {
    InvalidPlayer,
    PlayerEliminated,
    NotPlayersTurn,
    GameAlreadyFinished,
    InitialColorChoiceRequired,
    InitialColorAlreadyChosen,
    CardNotInHand,
    CardDoesNotMatch,
    ColorRequired,
    UnexpectedColor,
    MustPlayDrawnCard,
    MustResolveDrawPenalty,
    NoDrawPenalty,
    CannotStack,
    CannotChallenge,
    MustDrawBeforePassing,
    MustResolveSkip,
    NoSkipToResolve,
    UnoCalloutDisabled,
    CannotCallUno,
    MustPlayAfterUno,
    CannotReportSelf,
    PlayerNotReportable,
    CannotPlayTogether,
    CannotJumpIn,
    MustResolveSwapEffect,
    NoSwapEffect,
    InvalidSwapTargets,
    DrawPileExhausted,
}

impl fmt::Display for UnoViolation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::InvalidPlayer => "玩家身份无效",
            Self::PlayerEliminated => "你已经被淘汰，不能继续操作",
            Self::NotPlayersTurn => "还没有轮到你行动",
            Self::GameAlreadyFinished => "游戏已经结束",
            Self::InitialColorChoiceRequired => "请先为起始万能牌选择颜色",
            Self::InitialColorAlreadyChosen => "当前不需要选择起始颜色",
            Self::CardNotInHand => "这张牌不在你的手中",
            Self::CardDoesNotMatch => "这张牌与当前颜色、数字或符号不匹配",
            Self::ColorRequired => "万能牌必须选择后续颜色",
            Self::UnexpectedColor => "普通牌不能指定后续颜色",
            Self::MustPlayDrawnCard => "摸牌后只能打出刚摸到的牌",
            Self::MustResolveDrawPenalty => "请先叠加、质疑或接受累计罚牌",
            Self::NoDrawPenalty => "当前没有待结算的罚牌",
            Self::CannotStack => "这张牌不能叠加到当前罚牌上",
            Self::CannotChallenge => "当前没有可质疑的万能摸四",
            Self::MustDrawBeforePassing => "必须先摸牌，才能结束回合",
            Self::MustResolveSkip => "请先叠加禁手或接受累计禁手",
            Self::NoSkipToResolve => "当前没有待结算的禁手",
            Self::UnoCalloutDisabled => "本房间没有启用 UNO 宣告与检举",
            Self::CannotCallUno => "你当前不能宣告 UNO",
            Self::MustPlayAfterUno => "喊出 UNO 后，本回合必须出牌至只剩一张",
            Self::CannotReportSelf => "不能检举自己",
            Self::PlayerNotReportable => "该玩家当前不可被检举",
            Self::CannotPlayTogether => "这些牌当前不能一次打出",
            Self::CannotJumpIn => "抢出窗口已经关闭",
            Self::DrawPileExhausted => "摸牌堆已经耗尽",
            Self::MustResolveSwapEffect => "请先完成当前换牌效果",
            Self::NoSwapEffect => "当前没有待处理的换牌效果",
            Self::InvalidSwapTargets => "请选择符合要求且互不重复的玩家",
        })
    }
}
