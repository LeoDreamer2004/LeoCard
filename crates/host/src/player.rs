//! 房间参与者对各游戏公开的身份、档案与房间状态。

use crate::room::Participant;
use leocard_protocol::{LobbyPlayerView, McGuessPlayerView, PlayerPublicInfo, SeatId};

#[derive(Clone, Debug)]
pub(super) struct PublicPlayerMetadata {
    pub player: PlayerPublicInfo,
    pub seat: Option<SeatId>,
    pub ready: bool,
    pub connected: bool,
    pub auto_play: bool,
}

impl Participant {
    pub(super) fn public_metadata(&self) -> PublicPlayerMetadata {
        PublicPlayerMetadata {
            player: PlayerPublicInfo {
                id: self.id,
                profile_id: self.profile_id,
                name: self.name.clone(),
                avatar: self.avatar,
                reference_points: self.reference_points,
                completed_games: self.completed_games,
                game_profiles: self.game_profiles.clone(),
            },
            seat: self.seat,
            ready: self.ready,
            connected: (self.connected || self.is_bot) && !self.left,
            auto_play: self.auto_play,
        }
    }
}

impl From<PublicPlayerMetadata> for LobbyPlayerView {
    fn from(public: PublicPlayerMetadata) -> Self {
        Self {
            player: public.player,
            seat: public.seat,
            ready: public.ready,
            connected: public.connected,
        }
    }
}

impl From<PublicPlayerMetadata> for McGuessPlayerView {
    fn from(public: PublicPlayerMetadata) -> Self {
        Self {
            player: public.player,
            seat: public.seat.expect("active guessing players have seats"),
            ready: public.ready,
            connected: public.connected,
        }
    }
}
