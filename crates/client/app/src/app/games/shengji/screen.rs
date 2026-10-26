//! This game owns its screen resources, state reconciliation and view assembly.

use super::super::{GameLobbyContext, GameScreen, GameTableContext};
use super::{
    ShengjiAssets, ShengjiPresentationState, ShengjiScoreCaptureEffectState,
    ShengjiSettlementAnimation, ShengjiTableVisuals, ShengjiUiState, render_shengji_lobby,
    render_shengji_table,
};
use crate::app::runtime::ClientResource;
use crate::app::shell::SocialUiState;
use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use leocard_client::ClientPhaseRef;
use leocard_protocol::{GameSnapshot, LobbySnapshot, ShengjiSnapshot};
use leocard_ui::UiComposer;

#[derive(SystemParam)]
pub(crate) struct ShengjiScreen<'w> {
    assets: Res<'w, ShengjiAssets>,
    ui: ResMut<'w, ShengjiUiState>,
    score_capture: Res<'w, ShengjiScoreCaptureEffectState>,
    settlement: Res<'w, ShengjiSettlementAnimation>,
    presentation: Res<'w, ShengjiPresentationState>,
}

impl GameScreen for ShengjiScreen<'_> {
    type Snapshot = ShengjiSnapshot;

    fn reconcile(&mut self, client: Option<&ClientResource>, _social: &mut SocialUiState) {
        match client.map(|client| client.0.model().phase()) {
            Some(ClientPhaseRef::Playing(GameSnapshot::Shengji(game))) => self.ui.reconcile(game),
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
        render_shengji_lobby(
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
            form,
            assets,
            avatars,
            appearance,
            transition,
            table_materials,
            turn_border_materials,
            ..
        } = context;
        render_shengji_table(
            commands,
            root,
            client,
            game,
            &mut self.ui,
            social,
            chat,
            ShengjiTableVisuals {
                assets,
                game_assets: &self.assets,
                avatars,
                appearance,
                brightness: form.table_brightness,
                vignette: form.table_vignette,
                table_materials,
                turn_border_materials,
                start_game_transition: transition,
                score_capture: &self.score_capture,
                settlement: &self.settlement,
                presentation: &self.presentation,
            },
        );
    }
}
