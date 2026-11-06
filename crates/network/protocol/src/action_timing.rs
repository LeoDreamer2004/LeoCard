use crate::{GameKind, MatchId};
use leocard_game_common::PlayerId;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct ActionClock {
    pub player: PlayerId,
    pub elapsed_millis: u64,
}

/// Authoritative elapsed time, also refreshed on transport heartbeats.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct ActionTimingSnapshot {
    pub game: GameKind,
    pub match_id: MatchId,
    pub clocks: Vec<ActionClock>,
}
