use crate::{LobbyPlayer, MatchId, PlayerId, PlayerReferenceChange};
pub use leocard_blackjack::GameError as BlackjackViolation;
use leocard_blackjack::{
    AvailableActions, BlackjackAction, BlackjackCard, BlackjackRuleSet, HandState, Phase,
    PlayerStatistics,
};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct BlackjackSnapshot {
    pub match_id: MatchId,
    pub host_port: u16,
    pub you: PlayerId,
    pub host: PlayerId,
    pub rules: BlackjackRuleSet,
    pub round: u8,
    pub phase: Phase,
    pub players: Vec<BlackjackPlayerView>,
    pub dealer: Vec<BlackjackCard>,
    pub dealer_hidden: bool,
    pub current_player: Option<PlayerId>,
    pub current_hand: Option<u8>,
    pub available: AvailableActions,
    pub seconds_remaining: Option<u16>,
    pub match_complete: bool,
    pub reference_changes: Vec<PlayerReferenceChange>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct BlackjackPlayerView {
    pub player: LobbyPlayer,
    pub stack: u32,
    pub round_start_stack: u32,
    pub hands: Vec<HandState>,
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

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct BlackjackProfileStats {
    pub completed_games: u32,
    pub total_reference_delta: i64,
    pub total_chips_delta: i64,
    pub placement_counts: [u32; 6],
    pub statistics: PlayerStatistics,
}
