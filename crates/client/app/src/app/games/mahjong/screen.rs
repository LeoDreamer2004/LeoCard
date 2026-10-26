//! This game owns its screen resources, state reconciliation and view assembly.

use super::super::{GameLobbyContext, GameScreen, GameTableContext};
use super::{
    MahjongAssets, MahjongClaimPresentationState, MahjongFanCalculator, MahjongTableVisuals,
    MahjongTileMaterial, MahjongUiState, render_mahjong_lobby, render_mahjong_table,
};
use crate::app::runtime::ClientResource;
use crate::app::shell::SocialUiState;
use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use leocard_client::ClientPhaseRef;
use leocard_protocol::{GameSnapshot, LobbySnapshot, MahjongSnapshot};
use leocard_ui::UiComposer;

#[derive(SystemParam)]
pub(crate) struct MahjongScreen<'w> {
    assets: Res<'w, MahjongAssets>,
    ui: ResMut<'w, MahjongUiState>,
    fan_calculator: Res<'w, MahjongFanCalculator>,
    claim_presentation: Res<'w, MahjongClaimPresentationState>,
    tile_materials: ResMut<'w, Assets<MahjongTileMaterial>>,
}

impl GameScreen for MahjongScreen<'_> {
    type Snapshot = MahjongSnapshot;

    fn reconcile(&mut self, client: Option<&ClientResource>, _social: &mut SocialUiState) {
        match client.map(|client| client.0.model().phase()) {
            Some(ClientPhaseRef::Playing(GameSnapshot::Mahjong(game))) => self.ui.reconcile(game),
            _ => self.ui.clear(),
        }
    }

    fn render_lobby(
        &self,
        commands: &mut UiComposer,
        root: Entity,
        lobby: &LobbySnapshot,
        context: GameLobbyContext<'_>,
    ) {
        render_mahjong_lobby(
            commands,
            root,
            context.client,
            lobby,
            context.assets,
            context.avatars,
        );
    }

    fn render_table(
        &mut self,
        commands: &mut UiComposer,
        root: Entity,
        game: &Self::Snapshot,
        context: GameTableContext<'_>,
    ) {
        let GameTableContext {
            client,
            social,
            chat,
            developer_hand,
            form,
            assets,
            avatars,
            appearance,
            game_summary,
            transition,
            table_materials,
            ..
        } = context;
        let interaction_menu_open = social.interaction_menu_open;
        render_mahjong_table(
            commands,
            root,
            client,
            game,
            &mut self.ui,
            chat,
            interaction_menu_open,
            MahjongTableVisuals {
                assets,
                game_assets: &self.assets,
                avatars,
                developer_hand,
                appearance,
                brightness: form.table_brightness,
                vignette: form.table_vignette,
                table_materials,
                tile_materials: &mut self.tile_materials,
                start_game_transition: transition,
                game_summary,
                claim_presentation: &self.claim_presentation,
                show_wait_fans: self.fan_calculator.active,
            },
        );
    }
}
