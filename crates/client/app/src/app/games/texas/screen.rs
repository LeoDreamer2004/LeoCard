//! This game owns its screen resources, state reconciliation and view assembly.

use super::super::{GameLobbyContext, GameScreen, GameTableContext};
use super::{
    TexasChipTableState, TexasTableVisuals, TexasUiState, render_texas_lobby, render_texas_table,
};
use crate::app::presentation::TableCardMaterial;
use crate::app::runtime::ClientResource;
use crate::app::shell::SocialUiState;
use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use leocard_client::ClientPhaseRef;
use leocard_protocol::{GameSnapshot, LobbySnapshot, TexasSnapshot};
use leocard_ui::UiComposer;

#[derive(SystemParam)]
pub(crate) struct TexasScreen<'w> {
    ui: ResMut<'w, TexasUiState>,
    chips: Res<'w, TexasChipTableState>,
    card_materials: ResMut<'w, Assets<TableCardMaterial>>,
}

impl GameScreen for TexasScreen<'_> {
    type Snapshot = TexasSnapshot;

    fn requires_action(&self, game: &Self::Snapshot) -> bool {
        use leocard_protocol::TexasPhaseView;
        matches!(game.phase, TexasPhaseView::Betting { .. })
            && game
                .blind_to_post
                .map_or(game.current_player == Some(game.you), |blind| {
                    blind.player == game.you
                })
            && game.players.iter().any(|player| {
                player.player.id == game.you
                    && player.connected
                    && !player.auto_play
                    && !player.folded
                    && !player.all_in
            })
    }

    fn reconcile(&mut self, client: Option<&ClientResource>, _social: &mut SocialUiState) {
        let Some(client) = client else {
            self.ui.clear();
            return;
        };
        let model = client.0.model();
        match model.phase() {
            ClientPhaseRef::Playing(GameSnapshot::Texas(game)) => {
                self.ui
                    .reconcile(game, model.latest_revision(), model.rejection_serial())
            }
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
        render_texas_lobby(
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
        render_texas_table(
            commands,
            root,
            client,
            game,
            &mut self.ui,
            social,
            chat,
            TexasTableVisuals {
                assets,
                avatars,
                card_materials: &mut self.card_materials,
                brightness: form.table_brightness,
                custom_felt: appearance.cards.custom_felt.as_ref(),
                table_materials,
                turn_border_materials,
                start_game_transition: transition,
                chip_state: &self.chips,
                game_summary,
            },
        );
    }
}
