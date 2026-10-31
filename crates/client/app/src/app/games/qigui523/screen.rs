//! This game owns its screen resources, state reconciliation and view assembly.
use super::super::{GameLobbyContext, GameScreen, GameTableContext};
use super::{
    QiGui523Assets, QiGui523UiState, QiGuiClientState, TableVisualContext, render_qigui523_lobby,
    render_table,
};
use crate::app::presentation::PlayEffectState;
use crate::app::runtime::ClientResource;
use crate::app::shell::{ScoreCaptureEffectState, SocialUiState};
use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use leocard_client::ClientPhaseRef;
use leocard_protocol::{GameSnapshot, LobbySnapshot, QiGui523Snapshot};
use leocard_ui::UiComposer;

#[derive(SystemParam)]
pub(crate) struct QiGui523Screen<'w> {
    client_state: Res<'w, QiGuiClientState>,
    assets: Res<'w, QiGui523Assets>,
    ui: ResMut<'w, QiGui523UiState>,
    play_effect: Res<'w, PlayEffectState>,
    score_capture: Res<'w, ScoreCaptureEffectState>,
}

impl GameScreen for QiGui523Screen<'_> {
    type Snapshot = QiGui523Snapshot;

    fn requires_action(&self, game: &Self::Snapshot) -> bool {
        use leocard_protocol::QiGui523PhaseView;
        matches!(game.phase, QiGui523PhaseView::Playing)
            && game
                .trick
                .as_ref()
                .is_some_and(|trick| trick.current_player == game.you)
            && game.players.iter().any(|player| {
                player.player.id == game.you
                    && player.connected
                    && !player.auto_play
                    && player.hand_len > 0
            })
    }

    fn reconcile(&mut self, client: Option<&ClientResource>, _social: &mut SocialUiState) {
        match client.map(|client| client.0.model().phase()) {
            Some(ClientPhaseRef::Playing(GameSnapshot::QiGui523(game))) => self.ui.reconcile(game),
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
        render_qigui523_lobby(
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
            turn_border_materials,
            ..
        } = context;
        let mut visuals = TableVisualContext {
            client_state: &self.client_state,
            assets,
            game_assets: &self.assets,
            avatars,
            appearance,
            brightness: form.table_brightness,
            vignette: form.table_vignette,
            table_materials,
            game_summary,
            play_effect: &self.play_effect,
            score_capture: &self.score_capture,
            start_game_transition: transition,
            turn_border_materials,
        };
        render_table(
            commands,
            root,
            client,
            game,
            &self.ui,
            social,
            chat,
            developer_hand,
            &mut visuals,
        );
    }
}
