use super::lobby::render_lobby;
use super::state::GuandanUiState;
use super::table::{GuandanTable, render_table};
use crate::app::games::{GameLobbyContext, GameScreen, GameTableContext};
use crate::app::runtime::ClientResource;
use crate::app::shell::SocialUiState;
use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use leocard_client::ClientPhaseRef;
use leocard_protocol::{GameSnapshot, GuandanSnapshot, LobbySnapshot};
use leocard_ui::UiComposer;

#[derive(SystemParam)]
pub(crate) struct GuandanScreen<'w> {
    ui: ResMut<'w, GuandanUiState>,
}

impl GameScreen for GuandanScreen<'_> {
    type Snapshot = GuandanSnapshot;

    fn requires_action(&self, game: &Self::Snapshot) -> bool {
        game.result.is_none()
            && game.current == Some(game.you)
            && game
                .players
                .iter()
                .any(|player| player.player.id == game.you && player.connected && !player.auto_play)
    }

    fn reconcile(&mut self, client: Option<&ClientResource>, _social: &mut SocialUiState) {
        match client.map(|client| client.0.model().phase()) {
            Some(ClientPhaseRef::Playing(GameSnapshot::Guandan(game))) => self.ui.reconcile(game),
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
        render_lobby(
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
        render_table(
            commands,
            root,
            GuandanTable {
                client: context.client,
                game,
                ui: &mut self.ui,
                social: context.social,
                chat: context.chat,
                assets: context.assets,
                avatars: context.avatars,
                brightness: context.form.table_brightness,
                custom_felt: context.appearance.cards.custom_felt.as_ref(),
                table_materials: context.table_materials,
                turn_materials: context.turn_border_materials,
                summary: context.game_summary,
                transition: context.transition,
            },
        );
    }
}
