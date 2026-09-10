//! 顶层页面导航状态。
use super::super::{PlayerProfilePage, ProfileGameTab};

#[derive(Default)]
pub(crate) struct NavigationUiState {
    pub settings_open: bool,
    pub profile_open: bool,
    pub player_profile: Option<PlayerProfilePage>,
    pub profile_game_tab: ProfileGameTab,
    pub host_game_picker_open: bool,
}
