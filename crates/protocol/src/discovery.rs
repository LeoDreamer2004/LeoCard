use crate::{AvatarId, GameKind};
use serde::{Deserialize, Serialize};

/// Public room information, available without joining or occupying a player slot.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct RoomInfo {
    pub host_name: String,
    pub host_avatar: Option<AvatarId>,
    pub avatar_png: Option<Vec<u8>>,
    pub game: GameKind,
    pub playing: bool,
}
