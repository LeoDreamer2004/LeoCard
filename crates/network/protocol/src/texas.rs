use crate::{MatchId, PlayerPublicInfo, PlayerReferenceChange, SeatId};
use leocard_game_common::PlayerId;
use leocard_texas_holdem::{
    TexasBlindKind, TexasCard, TexasEvaluatedHand, TexasHandCategory, TexasSquidTransfer,
    TexasStreet,
};

use serde::{Deserialize, Serialize};
use std::fmt;

/// 面向单个德州扑克客户端的私有快照。
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct TexasSnapshot {
    pub match_id: MatchId,
    pub hand_number: u32,
    pub host_port: u16,
    pub you: PlayerId,
    pub host: PlayerId,
    pub players: Vec<TexasPlayerView>,
    pub your_hole_cards: Vec<TexasCard>,
    pub revealed_hands: Vec<TexasRevealedHand>,
    pub community: Vec<TexasCard>,
    /// 接收者弃牌所在下注轮已结束，且仍有至少两名玩家继续对局。
    pub spectator_available: bool,
    /// 接收者请求的胜率开关状态；其他玩家的设置不公开。
    pub spectator_win_rates_enabled: bool,
    /// 仅弃牌所在下注轮已结束、且仍有至少两名玩家继续对局的接收者可见。
    /// Some([]) 表示正在精确计算；不包含其他玩家底牌或未来牌堆顺序。
    pub spectator_equities: Option<Vec<TexasEquity>>,
    /// 尚未从权威牌堆发出的牌数；客户端可扣除仍扣置在桌面的公共牌数量来绘制牌堆。
    pub draw_pile_len: u16,
    pub dealer: PlayerId,
    pub small_blind: PlayerId,
    pub big_blind: PlayerId,
    pub current_player: Option<PlayerId>,
    pub blind_to_post: Option<TexasBlindView>,
    pub current_bet: u32,
    pub minimum_raise_to: u32,
    /// 接收方本次能执行的最高普通加注；None 表示不能完整加注。
    pub maximum_raise_to: Option<u32>,
    pub amount_to_call: u32,
    pub raise_allowed: bool,
    pub all_in_allowed: bool,
    pub pot: u32,
    pub phase: TexasPhaseView,
    pub squid: TexasSquidView,
}

/// Marker order gives every physical marker a stable identity across hands.
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct TexasSquidView {
    pub round: u32,
    pub markers: Vec<Option<PlayerId>>,
    pub settled: bool,
    pub transfers: Vec<TexasSquidTransfer>,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct TexasEquity {
    pub player: PlayerId,
    /// 万分比；平局按获胜人数均分，弃牌玩家为零。
    pub basis_points: u16,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct TexasBlindView {
    pub player: PlayerId,
    pub kind: TexasBlindKind,
    pub amount: u32,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct TexasPlayerView {
    pub player: PlayerPublicInfo,
    pub seat: SeatId,
    pub stack: u32,
    /// 本手开始时的筹码，用于在客户端显示本手盈亏。
    pub hand_start_stack: u32,
    pub committed_street: u32,
    pub committed_total: u32,
    pub folded: bool,
    pub all_in: bool,
    pub connected: bool,
    /// 由房主执行保守策略的托管状态；开发者模式补入的机器人始终为 `true`。
    pub auto_play: bool,
    /// 本手结算窗口中的下一手准备状态；房主不享有默认准备。
    pub ready: bool,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct TexasRevealedHand {
    pub player: PlayerId,
    /// 标准德州为两张，奥马哈为四张。
    pub cards: Vec<TexasCard>,
    /// 未发满五张公共牌便因其他玩家弃牌结束时，可能不足以组成五张牌型。
    pub best: Option<TexasEvaluatedHand>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum TexasPhaseView {
    Betting {
        street: TexasStreet,
    },
    HandComplete {
        showdown: bool,
        awards: Vec<TexasPotAwardView>,
        table_winner: Option<PlayerId>,
        /// 任意玩家筹码归零时，本场比赛立即结束。
        tournament_complete: bool,
        /// 仅整场结束时包含共享参考积分变化。
        reference_changes: Vec<PlayerReferenceChange>,
    },
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct TexasPotAwardView {
    pub amount: u32,
    pub winners: Vec<PlayerId>,
    pub winning_category: Option<TexasHandCategory>,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum TexasViolation {
    InvalidPlayer,
    NotPlayersTurn,
    HandAlreadyComplete,
    PlayerCannotAct,
    MustPostBlind,
    NoBlindToPost,
    CannotCheckWhileFacingBet { amount_to_call: u32 },
    NothingToCall,
    RaiseMustExceedCurrentBet { current_bet: u32, target: u32 },
    RaiseBelowMinimum { minimum_target: u32, target: u32 },
    RaiseExceedsStack { maximum_target: u32, target: u32 },
    RaiseNotReopened,
    RaiseExceedsLimit { maximum_target: u32, target: u32 },
    RaiseLimitReached,
}

impl fmt::Display for TexasViolation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::InvalidPlayer => "玩家身份无效",
            Self::NotPlayersTurn => "还没有轮到你行动",
            Self::HandAlreadyComplete => "这一手已经结束",
            Self::PlayerCannotAct => "弃牌、全下或离线玩家不能继续行动",
            Self::MustPostBlind => "当前只能下盲注",
            Self::NoBlindToPost => "当前没有需要下的盲注",
            Self::CannotCheckWhileFacingBet { .. } => "面对下注时不能过牌",
            Self::NothingToCall => "当前没有需要跟注的筹码",
            Self::RaiseMustExceedCurrentBet { .. } => "加注必须超过当前最高注",
            Self::RaiseBelowMinimum { .. } => "加注没有达到本轮最小额度",
            Self::RaiseExceedsStack { .. } => "加注额度超过了你的可用筹码",
            Self::RaiseNotReopened => "本轮下注尚未重新开放加注",
            Self::RaiseExceedsLimit { .. } => "加注超过了固定限注额度",
            Self::RaiseLimitReached => "本轮已达到三次加注上限",
        })
    }
}
