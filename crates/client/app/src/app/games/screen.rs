//! Game-independent inputs supplied by the application screen compositor.

use crate::app::presentation::{
    GameSummaryAnimation, StartGameSeatTransition, TableBackgroundMaterial, TurnBorderMaterial,
};
use crate::app::runtime::{
    AppearancePreferences, AvatarImages, ClientResource, TableAppearance, UiAssets,
};
use crate::app::shell::{ChatPanelState, DeveloperHandInput, SocialUiState};
use bevy::prelude::*;
use leocard_protocol::LobbySnapshot;
use leocard_ui::UiComposer;

pub(crate) struct GameLobbyContext<'a> {
    pub client: &'a ClientResource,
    pub assets: &'a UiAssets,
    pub avatars: &'a AvatarImages,
}

pub(crate) struct GameTableContext<'a> {
    pub client: &'a ClientResource,
    pub social: &'a SocialUiState,
    pub chat: &'a ChatPanelState,
    pub developer_hand: &'a DeveloperHandInput,
    pub form: &'a AppearancePreferences,
    pub assets: &'a UiAssets,
    pub avatars: &'a AvatarImages,
    pub appearance: &'a TableAppearance,
    pub game_summary: &'a GameSummaryAnimation,
    pub transition: &'a StartGameSeatTransition,
    pub table_materials: &'a mut Assets<TableBackgroundMaterial>,
    pub turn_border_materials: &'a mut Assets<TurnBorderMaterial>,
}

pub(crate) trait GameScreen {
    type Snapshot;

    /// 当前接收者是否需要作出游戏决策；不包含聊天、商店等常驻操作。
    fn requires_action(&self, game: &Self::Snapshot) -> bool;

    fn reconcile(&mut self, _client: Option<&ClientResource>, _social: &mut SocialUiState) {}

    fn render_lobby(
        &self,
        commands: &mut UiComposer,
        root: Entity,
        lobby: &LobbySnapshot,
        context: GameLobbyContext<'_>,
    );

    fn render_table(
        &mut self,
        commands: &mut UiComposer,
        root: Entity,
        game: &Self::Snapshot,
        context: GameTableContext<'_>,
    );
}
