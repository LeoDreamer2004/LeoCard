//! 各游戏共用的玩家公开状态与结算数据。

use crate::{
    AvatarId, BlackjackPlayerView, DoudizhuPlayerView, LobbyPlayerView, MahjongPlayerView,
    McGuessPlayerView, PlayerGameProfiles, ProfileId, QiGui523PlayerView, ShengjiPlayerView,
    TexasPlayerView, UnoPlayerView,
};
use leocard_game_common::PlayerId;
use serde::{Deserialize, Serialize};

/// Public identity and profile data, independent of seats and game rules.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct PlayerPublicInfo {
    pub id: PlayerId,
    pub profile_id: ProfileId,
    pub name: String,
    pub avatar: Option<AvatarId>,
    pub reference_points: i32,
    pub completed_games: u32,
    pub game_profiles: PlayerGameProfiles,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct PlayerScore {
    pub player: PlayerId,
    pub score: u32,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct PlayerReferenceChange {
    pub player: PlayerId,
    pub profile_id: ProfileId,
    pub delta: i16,
}

macro_rules! player_views {
    ($($view:ty),+ $(,)?) => {
        $(
            impl AsRef<PlayerPublicInfo> for $view {
                fn as_ref(&self) -> &PlayerPublicInfo {
                    &self.player
                }
            }
        )+
    };
}

player_views!(
    LobbyPlayerView,
    QiGui523PlayerView,
    DoudizhuPlayerView,
    TexasPlayerView,
    ShengjiPlayerView,
    UnoPlayerView,
    MahjongPlayerView,
    McGuessPlayerView,
    BlackjackPlayerView,
);

/// A borrowed projection of differently typed player lists; queries do not clone profiles.
pub(crate) trait PlayerRoster {
    fn player_count(&self) -> usize;

    fn player_info(&self, index: usize) -> &PlayerPublicInfo;
}

impl<T: AsRef<PlayerPublicInfo>> PlayerRoster for Vec<T> {
    fn player_count(&self) -> usize {
        self.len()
    }

    fn player_info(&self, index: usize) -> &PlayerPublicInfo {
        self[index].as_ref()
    }
}
