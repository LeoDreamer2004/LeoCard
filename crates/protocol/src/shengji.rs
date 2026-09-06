use crate::{
    AvatarId, MatchId, PlayerGameProfiles, PlayerId, PlayerReferenceChange, ProfileId, SeatId,
};
use leocard_shengji::{
    ShengjiBidKind, ShengjiBidTrump, ShengjiCard, ShengjiClassifiedPlay, ShengjiRank,
    ShengjiRuleSet, ShengjiTeamId, ShengjiTrump,
};
use serde::{Deserialize, Serialize};

/// 面向单个升级客户端生成的私有快照。
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct ShengjiSnapshot {
    pub match_id: MatchId,
    pub hand_number: u32,
    pub host_port: u16,
    pub you: PlayerId,
    pub host: PlayerId,
    pub rules: ShengjiRuleSet,
    pub players: Vec<ShengjiPlayerState>,
    pub your_hand: Vec<ShengjiCard>,
    /// 仅向接收者公开其本人此前亮过的物理牌；王在带王亮规则下可以复用，
    /// 级牌不可复用。用于重连后继续生成正确的亮主候选。
    pub your_exposed_cards: Vec<ShengjiCard>,
    pub levels: [ShengjiRank; 2],
    /// 本局抢亮所使用的级牌。首局无人亮主前庄家尚未公开；首局亮主后随当前
    /// 最高声明实时转移，后续小局则从发牌开始公开上一局已经确定的庄家。
    pub bidding_level: ShengjiRank,
    pub dealer: Option<PlayerId>,
    pub trump: Option<ShengjiTrump>,
    pub declaration: Option<ShengjiDeclarationView>,
    pub current_player: Option<PlayerId>,
    pub trick: Option<ShengjiTrickView>,
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
pub struct ShengjiPlayerState {
    pub id: PlayerId,
    pub profile_id: ProfileId,
    pub name: String,
    pub avatar: Option<AvatarId>,
    pub seat: SeatId,
    pub hand_len: u8,
    pub ready: bool,
    pub connected: bool,
    pub auto_play: bool,
    pub reference_points: i32,
    pub completed_games: u32,
    pub game_profiles: PlayerGameProfiles,
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
        reveal: Option<ShengjiBottomFlipRevealView>,
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

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum ShengjiFiveTrumpCrossingStage {
    Deciding,
    Returning,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct ShengjiBottomFlipMatchView {
    pub player: PlayerId,
    pub cards: Vec<ShengjiCard>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct ShengjiBottomFlipRevealView {
    pub card: ShengjiCard,
    pub matches: Vec<ShengjiBottomFlipMatchView>,
    pub dealer: Option<PlayerId>,
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
