use crate::{MatchId, PlayerPublicInfo, PlayerReferenceChange, PlayerScore, SeatId};
use leocard_game_common::PlayerId;
use leocard_mc_guess::{
    McGuessActionStatistics, McGuessGameError, McGuessGraphView, McGuessHandStatistics,
    McGuessMatchProgress, McGuessRoundView, McGuessRuleSet, McGuessScoreGain,
};
use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct McGuessSnapshot {
    pub match_id: MatchId,
    pub host_port: u16,
    pub you: PlayerId,
    pub host: PlayerId,
    pub players: Vec<McGuessPlayerView>,
    pub rules: McGuessRuleSet,
    pub progress: McGuessMatchProgress,
    pub round: McGuessRoundView,
    pub graph: Option<McGuessGraphView>,
    pub guess_seconds_remaining: Option<u16>,
    pub scores: Vec<PlayerScore>,
    pub score_gains: Vec<McGuessScoreGain>,
    pub reference_changes: Vec<PlayerReferenceChange>,
    /// Advisory presence only; never affects guessing rules or scoring.
    pub mc_process_players: Vec<PlayerId>,
}

impl McGuessSnapshot {
    pub fn can_guess(&self) -> bool {
        match &self.graph {
            Some(graph) => self.can_guess_node(graph.selected),
            None => self.is_guessing_player(),
        }
    }

    /// Node eligibility independent of which question the host has selected.
    pub fn can_guess_node(&self, number: u16) -> bool {
        self.is_guessing_player()
            && self
                .graph
                .as_ref()
                .is_some_and(|graph| graph.can_guess_node(number))
    }

    fn is_guessing_player(&self) -> bool {
        self.round.can_guess(self.you, self.rules.mode)
            && self
                .players
                .iter()
                .any(|player| player.player.id == self.you)
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum McGuessEvent {
    ActionAnalyzed {
        player: PlayerId,
        statistics: Box<McGuessActionStatistics>,
    },
    HandAnalyzed {
        player: PlayerId,
        statistics: McGuessHandStatistics,
    },
    ProcessPresenceChanged {
        player: PlayerId,
        name: String,
        running: bool,
    },
}

/// Client-facing rejection reasons, independent of the rules engine error type.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum McGuessViolation {
    NotPlayersTurn,
    CannotSkip,
    Finished,
    UnknownEntry,
    AlreadyGuessed,
    InvalidGuess,
    GuessTooLong,
    GraphUnavailable,
    InvalidNode,
    NodeSwitchCoolingDown,
    CannotGuessTag,
    NodeAlreadyRevealed,
    ComboRequiresNeighbour,
}

impl From<McGuessGameError> for McGuessViolation {
    fn from(value: McGuessGameError) -> Self {
        match value {
            McGuessGameError::NotPlayersTurn => Self::NotPlayersTurn,
            McGuessGameError::CannotSkip => Self::CannotSkip,
            McGuessGameError::Finished => Self::Finished,
            McGuessGameError::UnknownEntry => Self::UnknownEntry,
            McGuessGameError::AlreadyGuessed => Self::AlreadyGuessed,
            McGuessGameError::InvalidGuess => Self::InvalidGuess,
            McGuessGameError::GuessTooLong => Self::GuessTooLong,
            McGuessGameError::GraphUnavailable => Self::GraphUnavailable,
            McGuessGameError::InvalidNode => Self::InvalidNode,
            McGuessGameError::NodeSwitchCoolingDown => Self::NodeSwitchCoolingDown,
            McGuessGameError::CannotGuessTag => Self::CannotGuessTag,
            McGuessGameError::NodeAlreadyRevealed => Self::NodeAlreadyRevealed,
            McGuessGameError::ComboRequiresNeighbour => Self::ComboRequiresNeighbour,
        }
    }
}

impl From<McGuessViolation> for McGuessGameError {
    fn from(value: McGuessViolation) -> Self {
        match value {
            McGuessViolation::NotPlayersTurn => Self::NotPlayersTurn,
            McGuessViolation::CannotSkip => Self::CannotSkip,
            McGuessViolation::Finished => Self::Finished,
            McGuessViolation::UnknownEntry => Self::UnknownEntry,
            McGuessViolation::AlreadyGuessed => Self::AlreadyGuessed,
            McGuessViolation::InvalidGuess => Self::InvalidGuess,
            McGuessViolation::GuessTooLong => Self::GuessTooLong,
            McGuessViolation::GraphUnavailable => Self::GraphUnavailable,
            McGuessViolation::InvalidNode => Self::InvalidNode,
            McGuessViolation::NodeSwitchCoolingDown => Self::NodeSwitchCoolingDown,
            McGuessViolation::CannotGuessTag => Self::CannotGuessTag,
            McGuessViolation::NodeAlreadyRevealed => Self::NodeAlreadyRevealed,
            McGuessViolation::ComboRequiresNeighbour => Self::ComboRequiresNeighbour,
        }
    }
}

impl fmt::Display for McGuessViolation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        McGuessGameError::from(*self).fmt(f)
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct McGuessPlayerView {
    pub player: PlayerPublicInfo,
    pub seat: SeatId,
    pub ready: bool,
    pub connected: bool,
}
