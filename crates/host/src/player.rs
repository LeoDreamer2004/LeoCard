//! 房间参与者对各游戏公开的稳定身份与档案数据。

use crate::room::Participant;
use leocard_protocol::{AvatarId, LobbyPlayer, PlayerGameProfiles, PlayerId, ProfileId, SeatId};

#[derive(Clone, Debug)]
pub(super) struct PublicPlayerMetadata {
    pub id: PlayerId,
    pub profile_id: ProfileId,
    pub name: String,
    pub avatar: Option<AvatarId>,
    pub seat: Option<SeatId>,
    pub ready: bool,
    pub connected: bool,
    pub auto_play: bool,
    pub reference_points: i32,
    pub completed_games: u32,
    pub game_profiles: PlayerGameProfiles,
}

impl Participant {
    pub(super) fn public_metadata(&self) -> PublicPlayerMetadata {
        PublicPlayerMetadata {
            id: self.id,
            profile_id: self.profile_id,
            name: self.name.clone(),
            avatar: self.avatar,
            seat: self.seat,
            ready: self.ready,
            connected: (self.connected || self.is_bot) && !self.left,
            auto_play: self.auto_play,
            reference_points: self.reference_points,
            completed_games: self.completed_games,
            game_profiles: self.game_profiles.clone(),
        }
    }
}

impl From<PublicPlayerMetadata> for LobbyPlayer {
    fn from(player: PublicPlayerMetadata) -> Self {
        Self {
            id: player.id,
            profile_id: player.profile_id,
            name: player.name,
            avatar: player.avatar,
            seat: player.seat,
            ready: player.ready,
            connected: player.connected,
            reference_points: player.reference_points,
            completed_games: player.completed_games,
            game_profiles: player.game_profiles,
        }
    }
}
