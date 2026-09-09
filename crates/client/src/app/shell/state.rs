//! 顶层 UI 状态，仅组合各功能域自行维护的局部状态。

use super::{NavigationUiState, SocialUiState};
use bevy::prelude::*;

#[derive(Resource, Default)]
pub(crate) struct UiState {
    pub navigation: NavigationUiState,
    pub social: SocialUiState,
    pub leaving_room: bool,
    pub dirty: bool,
}
