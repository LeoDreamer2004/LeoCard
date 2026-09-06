use leocard_uno::{UnoCard, UnoColor, UnoDirection, UnoFlipSide, UnoPendingDrawKind, UnoRuleSet};
use serde::{Deserialize, Serialize};

use crate::{
    AvatarId, MatchId, PlayerGameProfiles, PlayerId, PlayerReferenceChange, ProfileId, SeatId,
};

/// 面向单个 UNO 客户端的私有快照。
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct UnoSnapshot {
    pub match_id: MatchId,
    pub host_port: u16,
    pub you: PlayerId,
    pub host: PlayerId,
    pub rules: UnoRuleSet,
    pub players: Vec<UnoPlayerState>,
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
pub struct UnoPlayerState {
    pub id: PlayerId,
    pub profile_id: ProfileId,
    pub name: String,
    pub avatar: Option<AvatarId>,
    pub seat: SeatId,
    pub hand_len: u8,
    /// FLIP 模式公开的手牌背面，按当前手牌顺序紧密排列。
    pub inactive_hand: Vec<UnoCard>,
    pub ready: bool,
    pub connected: bool,
    pub auto_play: bool,
    pub reference_points: i32,
    pub completed_games: u32,
    pub game_profiles: PlayerGameProfiles,
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
