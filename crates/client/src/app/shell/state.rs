//! 顶层 UI 状态，仅组合各功能域自行维护的局部状态。

use bevy::prelude::*;

use super::*;

#[derive(Resource, Default)]
pub struct UiState {
    pub qigui523: QiGui523UiState,
    pub texas_holdem: TexasHoldemUiState,
    pub shengji: ShengjiUiState,
    pub uno: UnoUiState,
    pub mahjong: MahjongUiState,
    pub navigation: NavigationUiState,
    pub social: SocialUiState,
    pub leaving_room: bool,
    pub dirty: bool,
}
