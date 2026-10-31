use crate::{MatchId, PlayerPublicInfo, PlayerReferenceChange, SeatId};
use leocard_doudizhu::achievements::DoudizhuAchievementFacts;
use leocard_doudizhu::{
    DoudizhuAction, DoudizhuCard, DoudizhuGameError, DoudizhuMatchProgress, DoudizhuPhase,
    DoudizhuPlay, DoudizhuResultView, DoudizhuRuleSet,
};
use leocard_game_common::PlayerId;
use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum DoudizhuCommand {
    UpdateRules { rules: DoudizhuRuleSet },
    SetAutoPlay { enabled: bool },
    Act(DoudizhuAction),
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum DoudizhuEvent {
    ActionApplied {
        player: PlayerId,
        action: DoudizhuAction,
    },
    Redealt,
    HandFinished {
        result: DoudizhuResultView,
    },
    ActionAnalyzed {
        player: PlayerId,
        statistics: Box<DoudizhuAchievementFacts>,
    },
}

/// Recipient-specific state. Other hands are sent only after settlement.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct DoudizhuSnapshot {
    pub match_id: MatchId,
    pub host_port: u16,
    pub you: PlayerId,
    pub host: PlayerId,
    pub players: Vec<DoudizhuPlayerView>,
    pub rules: DoudizhuRuleSet,
    pub progress: DoudizhuMatchProgress,
    pub scores: Vec<i32>,
    pub match_complete: bool,
    pub reference_changes: Vec<PlayerReferenceChange>,
    pub phase: DoudizhuPhase,
    pub current: PlayerId,
    pub landlord: Option<PlayerId>,
    pub highest_bid: u8,
    pub bids: Vec<Option<u8>>,
    pub your_hand: Vec<DoudizhuCard>,
    pub kitty: Vec<DoudizhuCard>,
    pub played: Vec<DoudizhuCard>,
    pub table: Option<(PlayerId, DoudizhuPlay)>,
    pub last_actions: Vec<Option<DoudizhuAction>>,
    pub multiplier: u32,
    pub result: Option<DoudizhuResultView>,
    pub revealed_hands: Vec<Vec<DoudizhuCard>>,
}

/// Client-facing rejection reasons, independent of the rules engine error type.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum DoudizhuViolation {
    InvalidRules,
    InvalidDeck,
    InvalidPlayer,
    NotYourTurn,
    WrongPhase,
    InvalidBid,
    InvalidPattern,
    CardNotInHand,
    MustPlay,
    CannotBeat,
}

impl From<DoudizhuGameError> for DoudizhuViolation {
    fn from(value: DoudizhuGameError) -> Self {
        match value {
            DoudizhuGameError::InvalidRules => Self::InvalidRules,
            DoudizhuGameError::InvalidDeck => Self::InvalidDeck,
            DoudizhuGameError::InvalidPlayer => Self::InvalidPlayer,
            DoudizhuGameError::NotYourTurn => Self::NotYourTurn,
            DoudizhuGameError::WrongPhase => Self::WrongPhase,
            DoudizhuGameError::InvalidBid => Self::InvalidBid,
            DoudizhuGameError::InvalidPattern => Self::InvalidPattern,
            DoudizhuGameError::CardNotInHand => Self::CardNotInHand,
            DoudizhuGameError::MustPlay => Self::MustPlay,
            DoudizhuGameError::CannotBeat => Self::CannotBeat,
        }
    }
}

impl From<DoudizhuViolation> for DoudizhuGameError {
    fn from(value: DoudizhuViolation) -> Self {
        match value {
            DoudizhuViolation::InvalidRules => Self::InvalidRules,
            DoudizhuViolation::InvalidDeck => Self::InvalidDeck,
            DoudizhuViolation::InvalidPlayer => Self::InvalidPlayer,
            DoudizhuViolation::NotYourTurn => Self::NotYourTurn,
            DoudizhuViolation::WrongPhase => Self::WrongPhase,
            DoudizhuViolation::InvalidBid => Self::InvalidBid,
            DoudizhuViolation::InvalidPattern => Self::InvalidPattern,
            DoudizhuViolation::CardNotInHand => Self::CardNotInHand,
            DoudizhuViolation::MustPlay => Self::MustPlay,
            DoudizhuViolation::CannotBeat => Self::CannotBeat,
        }
    }
}

impl fmt::Display for DoudizhuViolation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        DoudizhuGameError::from(*self).fmt(f)
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct DoudizhuPlayerView {
    pub player: PlayerPublicInfo,
    pub seat: SeatId,
    pub hand_len: u16,
    pub ready: bool,
    pub connected: bool,
    /// 由房主执行的托管状态；所有客户端都可见。
    pub auto_play: bool,
}
