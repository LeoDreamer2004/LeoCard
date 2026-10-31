use crate::{MatchId, PlayerPublicInfo, PlayerReferenceChange, PlayerScore, SeatId};
use leocard_game_common::PlayerId;
use leocard_qigui523::{QiGuiCard, QiGuiPlayKind};
use serde::{Deserialize, Serialize};
use std::fmt;

/// 面向单个七鬼五二三客户端生成的状态。
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct QiGui523Snapshot {
    pub match_id: MatchId,
    pub host_port: u16,
    pub you: PlayerId,
    pub host: PlayerId,
    pub players: Vec<QiGui523PlayerView>,
    pub your_hand: Vec<QiGuiCard>,
    /// 本局实际打出的公开牌，含已经结束的轮次。
    pub played_cards: Vec<QiGuiCard>,
    pub draw_pile_len: u16,
    pub starting_card: QiGui523StartingCardView,
    pub trick: Option<QiGui523TrickView>,
    pub turn_timer: Option<QiGui523TurnTimerView>,
    pub phase: QiGui523PhaseView,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct QiGui523TurnTimerView {
    pub player: PlayerId,
    pub base_seconds: u16,
    pub reserve_seconds: u16,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct QiGui523StartingCardView {
    pub player: PlayerId,
    pub card: QiGuiCard,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct QiGui523TrickView {
    pub leader: PlayerId,
    pub current_player: PlayerId,
    pub winning_player: Option<PlayerId>,
    pub winning_play: Option<QiGui523PublicPlay>,
    pub records: Vec<QiGui523PublicPlayRecord>,
    pub table_points: u32,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct QiGui523PublicPlay {
    pub kind: QiGuiPlayKind,
    pub cards: Vec<QiGuiCard>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum QiGui523PublicPlayRecord {
    Played {
        player: PlayerId,
        play: QiGui523PublicPlay,
    },
    Passed {
        player: PlayerId,
    },
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum QiGui523PhaseView {
    Playing,
    Finished {
        match_id: MatchId,
        finisher: PlayerId,
        scores: Vec<PlayerScore>,
        remaining_hands: Vec<QiGui523RevealedHand>,
        reference_changes: Vec<PlayerReferenceChange>,
        captured_hand_points: u32,
    },
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct QiGui523RevealedHand {
    pub player: PlayerId,
    pub cards: Vec<QiGuiCard>,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum QiGui523Violation {
    InvalidDeveloperHand,
    InvalidPlayer,
    NotPlayersTurn,
    GameAlreadyFinished,
    MustLeadWithCards,
    CardNotInHand,
    InvalidPattern,
    PlayDoesNotBeatCurrent,
}

impl fmt::Display for QiGui523Violation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::InvalidDeveloperHand => "开发者手牌无效",
            Self::NotPlayersTurn => "还没有轮到你行动",
            Self::MustLeadWithCards => "领出时必须出牌",
            Self::CardNotInHand => "选择的牌不在手中",
            Self::InvalidPattern => "所选牌不能组成合法牌型",
            Self::PlayDoesNotBeatCurrent => "这手牌无法压过当前牌",
            Self::GameAlreadyFinished => "游戏已经结束",
            Self::InvalidPlayer => "玩家身份无效",
        })
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct QiGui523PlayerView {
    pub player: PlayerPublicInfo,
    pub seat: SeatId,
    pub hand_len: u16,
    pub score: u32,
    pub ready: bool,
    pub connected: bool,
    /// 由房主执行的托管状态；所有客户端都可见。
    pub auto_play: bool,
}
