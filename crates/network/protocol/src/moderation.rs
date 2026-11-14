use crate::PlayerPublicInfo;
use leocard_game_common::PlayerId;
use serde::{Deserialize, Serialize};

/// Room controls and roster shared by the lobby, game and spectator clients.
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct RoomModerationSnapshot {
    pub host: Option<PlayerId>,
    pub spectator_interactions_blocked: bool,
    pub participants: Vec<RoomParticipant>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct RoomParticipant {
    pub player: PlayerPublicInfo,
    pub spectating: bool,
    pub connected: bool,
    pub is_bot: bool,
}
