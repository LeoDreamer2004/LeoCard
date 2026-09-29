//! 顶层页面导航状态。
use super::super::{PlayerProfilePage, ProfileGameTab, SettingsTab};

#[derive(Default)]
pub(crate) struct NavigationUiState {
    pub settings_open: bool,
    pub settings_tab: SettingsTab,
    pub profile_open: bool,
    pub player_profile: Option<PlayerProfilePage>,
    pub profile_game_tab: ProfileGameTab,
}
