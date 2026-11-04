//! 顶层 UI 状态，仅组合各功能域自行维护的局部状态。

use super::connection::{HomeUiState, LanUiState};
use super::{AchievementsUiState, ProfileUiState, SettingsUiState, ShopUiState, SocialUiState};
use bevy::prelude::*;

#[derive(Resource, Default)]
pub(crate) struct UiState {
    pub home: HomeUiState,
    pub achievements: AchievementsUiState,
    pub lan: LanUiState,
    pub shop: ShopUiState,
    pub profile: ProfileUiState,
    pub settings: SettingsUiState,
    pub social: SocialUiState,
    pub leaving_room: bool,
    pub dirty: bool,
}
