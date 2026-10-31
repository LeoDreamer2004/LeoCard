use crate::{MatchId, PlayerPublicInfo, PlayerReferenceChange, SeatId};
use leocard_blackjack::{
    BlackjackAction, BlackjackAvailableActions, BlackjackCard, BlackjackGameError,
    BlackjackHandState, BlackjackPhase, BlackjackRuleSet,
};
use leocard_game_common::PlayerId;
use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct BlackjackSnapshot {
    pub match_id: MatchId,
    pub host_port: u16,
    pub you: PlayerId,
    pub host: PlayerId,
    pub rules: BlackjackRuleSet,
    pub round: u8,
    pub phase: BlackjackPhase,
    pub players: Vec<BlackjackPlayerView>,
    pub dealer: Vec<BlackjackCard>,
    pub dealer_hidden: bool,
    pub current_player: Option<PlayerId>,
    pub current_hand: Option<u8>,
    pub available: BlackjackAvailableActions,
    pub seconds_remaining: Option<u16>,
    pub match_complete: bool,
    pub reference_changes: Vec<PlayerReferenceChange>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct BlackjackPlayerView {
    pub player: PlayerPublicInfo,
    pub seat: SeatId,
    pub ready: bool,
    pub connected: bool,
    pub stack: u32,
    pub round_start_stack: u32,
    pub hands: Vec<BlackjackHandState>,
    pub auto_play: bool,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum BlackjackCommand {
    UpdateRules { rules: BlackjackRuleSet },
    SetAutoPlay { enabled: bool },
    Act { action: BlackjackAction },
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum BlackjackEvent {
    ActionApplied {
        player: PlayerId,
        action: BlackjackAction,
    },
    RoundFinished {
        round: u8,
    },
}

/// Client-facing rejection reasons, independent of the rules engine error type.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum BlackjackViolation {
    InvalidRules,
    InvalidPlayerCount,
    InvalidShoe,
    NotPlayersTurn,
    WrongPhase,
    InvalidBet,
    CannotDouble,
    CannotSplit,
    MatchComplete,
}

impl From<BlackjackGameError> for BlackjackViolation {
    fn from(value: BlackjackGameError) -> Self {
        match value {
            BlackjackGameError::InvalidRules => Self::InvalidRules,
            BlackjackGameError::InvalidPlayerCount => Self::InvalidPlayerCount,
            BlackjackGameError::InvalidShoe => Self::InvalidShoe,
            BlackjackGameError::NotPlayersTurn => Self::NotPlayersTurn,
            BlackjackGameError::WrongPhase => Self::WrongPhase,
            BlackjackGameError::InvalidBet => Self::InvalidBet,
            BlackjackGameError::CannotDouble => Self::CannotDouble,
            BlackjackGameError::CannotSplit => Self::CannotSplit,
            BlackjackGameError::MatchComplete => Self::MatchComplete,
        }
    }
}

impl From<BlackjackViolation> for BlackjackGameError {
    fn from(value: BlackjackViolation) -> Self {
        match value {
            BlackjackViolation::InvalidRules => Self::InvalidRules,
            BlackjackViolation::InvalidPlayerCount => Self::InvalidPlayerCount,
            BlackjackViolation::InvalidShoe => Self::InvalidShoe,
            BlackjackViolation::NotPlayersTurn => Self::NotPlayersTurn,
            BlackjackViolation::WrongPhase => Self::WrongPhase,
            BlackjackViolation::InvalidBet => Self::InvalidBet,
            BlackjackViolation::CannotDouble => Self::CannotDouble,
            BlackjackViolation::CannotSplit => Self::CannotSplit,
            BlackjackViolation::MatchComplete => Self::MatchComplete,
        }
    }
}

impl fmt::Display for BlackjackViolation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        BlackjackGameError::from(*self).fmt(f)
    }
}
