use crate::{MatchId, PlayerPublicInfo, PlayerReferenceChange, SeatId};
use leocard_game_common::PlayerId;
use leocard_mahjong_link::{
    MahjongLinkAction, MahjongLinkBoard, MahjongLinkError, MahjongLinkOutcome, MahjongLinkRuleSet,
};
use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct MahjongLinkSnapshot {
    pub match_id: MatchId,
    pub host_port: u16,
    pub you: PlayerId,
    pub host: PlayerId,
    pub rules: MahjongLinkRuleSet,
    pub board: MahjongLinkBoard,
    pub players: Vec<MahjongLinkPlayerView>,
    pub current_player: Option<PlayerId>,
    pub turn: u32,
    pub push_used: bool,
    pub consecutive_passes: u8,
    pub match_complete: bool,
    pub last_action: Option<MahjongLinkEvent>,
    pub reference_changes: Vec<PlayerReferenceChange>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct MahjongLinkPlayerView {
    pub player: PlayerPublicInfo,
    pub seat: SeatId,
    pub ready: bool,
    pub connected: bool,
    pub auto_play: bool,
    pub score: u16,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum MahjongLinkCommand {
    SetAutoPlay { enabled: bool },
    Act { action: MahjongLinkAction },
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct MahjongLinkEvent {
    pub player: PlayerId,
    pub turn: u32,
    pub outcome: MahjongLinkOutcome,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct MahjongLinkViolation(pub MahjongLinkError);

impl From<MahjongLinkError> for MahjongLinkViolation {
    fn from(error: MahjongLinkError) -> Self {
        Self(error)
    }
}

impl fmt::Display for MahjongLinkViolation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}
