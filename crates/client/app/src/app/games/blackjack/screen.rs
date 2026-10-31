//! This game owns its screen resources, state reconciliation and view assembly.

use super::super::{GameLobbyContext, GameScreen, GameTableContext};
use super::{
    BlackjackPlayback, BlackjackTable, BlackjackUiState, render_blackjack_lobby,
    render_blackjack_table,
};
use crate::app::runtime::ClientResource;
use crate::app::shell::SocialUiState;
use bevy::ecs::system::SystemParam;
use leocard_client::ClientPhaseRef;
use leocard_protocol::{BlackjackSnapshot, GameSnapshot, LobbySnapshot};
use leocard_ui::UiComposer;

use bevy::prelude::*;

#[derive(SystemParam)]
pub(crate) struct BlackjackScreen<'w> {
    ui: ResMut<'w, BlackjackUiState>,
    playback: Res<'w, BlackjackPlayback>,
}

impl GameScreen for BlackjackScreen<'_> {
    type Snapshot = BlackjackSnapshot;

    fn reconcile(&mut self, client: Option<&ClientResource>, _social: &mut SocialUiState) {
        match client.map(|client| client.0.model().phase()) {
            Some(ClientPhaseRef::Playing(GameSnapshot::Blackjack(game))) => self.ui.reconcile(game),
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
        render_blackjack_lobby(
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
            game_summary,
            transition,
            table_materials,
            turn_border_materials,
            ..
        } = context;
        render_blackjack_table(
            commands,
            root,
            BlackjackTable {
                client,
                game,
                ui: &self.ui,
                playback: &self.playback,
                social,
                chat,
                assets,
                avatars,
                appearance,
                brightness: form.table_brightness,
                vignette: form.table_vignette,
                table_materials,
                turn_materials: turn_border_materials,
                transition,
                summary: game_summary,
            },
        );
    }
}
