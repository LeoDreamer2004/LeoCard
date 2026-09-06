use crate::{
    AvatarId, MatchId, PlayerGameProfiles, PlayerId, PlayerReferenceChange, ProfileId, SeatId,
};
use leocard_texas_holdem::{
    EvaluatedHand, TexasHoldemBlindKind, TexasHoldemCard, TexasHoldemHandCategory,
    TexasHoldemStreet,
};
use serde::{Deserialize, Serialize};

/// 面向单个德州扑克客户端的私有快照。
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct TexasHoldemSnapshot {
    pub match_id: MatchId,
    pub hand_number: u32,
    pub host_port: u16,
    pub you: PlayerId,
    pub host: PlayerId,
    pub players: Vec<TexasHoldemPlayerState>,
    pub your_hole_cards: Vec<TexasHoldemCard>,
    pub revealed_hands: Vec<TexasHoldemRevealedHand>,
    pub community: Vec<TexasHoldemCard>,
    /// 尚未从权威牌堆发出的牌数；客户端可扣除仍扣置在桌面的公共牌数量来绘制牌堆。
    pub draw_pile_len: u16,
    pub dealer: PlayerId,
    pub small_blind: PlayerId,
    pub big_blind: PlayerId,
    pub current_player: Option<PlayerId>,
    pub blind_to_post: Option<TexasHoldemBlindView>,
    pub current_bet: u32,
    pub minimum_raise_to: u32,
    pub amount_to_call: u32,
    pub raise_allowed: bool,
    pub pot: u32,
    pub phase: TexasHoldemPhaseView,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct TexasHoldemBlindView {
    pub player: PlayerId,
    pub kind: TexasHoldemBlindKind,
    pub amount: u32,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct TexasHoldemPlayerState {
    pub id: PlayerId,
    pub profile_id: ProfileId,
    pub name: String,
    pub avatar: Option<AvatarId>,
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
    pub reference_points: i32,
    pub completed_games: u32,
    pub game_profiles: PlayerGameProfiles,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct TexasHoldemRevealedHand {
    pub player: PlayerId,
    /// 标准德州为两张，奥马哈为四张。
    pub cards: Vec<TexasHoldemCard>,
    /// 未发满五张公共牌便因其他玩家弃牌结束时，可能不足以组成五张牌型。
    pub best: Option<EvaluatedHand>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum TexasHoldemPhaseView {
    Betting {
        street: TexasHoldemStreet,
    },
    HandComplete {
        showdown: bool,
        awards: Vec<TexasHoldemPotAward>,
        table_winner: Option<PlayerId>,
        /// 任意玩家筹码归零时，本场比赛立即结束。
        tournament_complete: bool,
        /// 仅整场结束时包含共享参考积分变化。
        reference_changes: Vec<PlayerReferenceChange>,
    },
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct TexasHoldemPotAward {
    pub amount: u32,
    pub winners: Vec<PlayerId>,
    pub winning_category: Option<TexasHoldemHandCategory>,
}
