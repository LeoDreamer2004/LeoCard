//! This game owns its screen resources, state reconciliation and view assembly.

use super::super::{GameLobbyContext, GameScreen, GameTableContext};
use super::{
    TexasChipTableState, TexasTableVisuals, TexasUiState, render_texas_lobby, render_texas_table,
};
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
        match client.map(|client| client.0.model().phase()) {
            Some(ClientPhaseRef::Playing(GameSnapshot::Texas(game))) => self.ui.reconcile(game),
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
                appearance,
                brightness: form.table_brightness,
                vignette: form.table_vignette,
                table_materials,
                turn_border_materials,
                start_game_transition: transition,
                chip_state: &self.chips,
                game_summary,
            },
        );
    }
}
