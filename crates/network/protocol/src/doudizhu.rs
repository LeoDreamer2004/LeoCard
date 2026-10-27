use crate::{MatchId, PlayerId, PlayerPublicState, PlayerReferenceChange};
use leocard_doudizhu::{Action, Card, MatchProgress, Phase, Play, ResultView, RuleSet};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum DoudizhuCommand {
    UpdateRules { rules: RuleSet },
    SetAutoPlay { enabled: bool },
    Act(Action),
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum DoudizhuEvent {
    ActionApplied { player: PlayerId, action: Action },
    Redealt,
    HandFinished { result: ResultView },
}

/// Recipient-specific state. Other hands are sent only after settlement.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct DoudizhuSnapshot {
    pub match_id: MatchId,
    pub host_port: u16,
    pub you: PlayerId,
    pub host: PlayerId,
    pub players: Vec<PlayerPublicState>,
    pub rules: RuleSet,
    pub progress: MatchProgress,
    pub scores: Vec<i32>,
    pub match_complete: bool,
    pub reference_changes: Vec<PlayerReferenceChange>,
    pub phase: Phase,
    pub current: PlayerId,
    pub landlord: Option<PlayerId>,
    pub highest_bid: u8,
    pub bids: Vec<Option<u8>>,
    pub your_hand: Vec<Card>,
    pub kitty: Vec<Card>,
    pub played: Vec<Card>,
    pub table: Option<(PlayerId, Play)>,
    pub last_actions: Vec<Option<Action>>,
    pub multiplier: u32,
    pub result: Option<ResultView>,
    pub revealed_hands: Vec<Vec<Card>>,
}
