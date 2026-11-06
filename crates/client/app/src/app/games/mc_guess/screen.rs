//! This game owns its screen resources, state reconciliation and view assembly.

use super::super::{GameLobbyContext, GameScreen, GameTableContext};
use super::{
    GraphArt, GuessFeedback, GuessScorePlayback, GuessTable, McGuessAssets, McGuessUiState,
    render_guess_lobby, render_guess_table,
};
use crate::app::games::{GameActionPrompt, GameUiLifecycle};
use crate::app::runtime::ClientResource;
use crate::app::shell::SocialUiState;
use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use leocard_protocol::{GameSnapshot, LobbySnapshot, McGuessSnapshot};
use leocard_ui::UiComposer;

#[derive(SystemParam)]
pub(crate) struct McGuessScreen<'w> {
    art: Res<'w, GraphArt>,
    assets: Res<'w, McGuessAssets>,
    ui: ResMut<'w, McGuessUiState>,
    pub(super) scores: Res<'w, GuessScorePlayback>,
    pub(super) feedback: Res<'w, GuessFeedback>,
}

impl GameScreen for McGuessScreen<'_> {
    type Snapshot = McGuessSnapshot;

    fn render_lobby(
        &self,
        commands: &mut UiComposer,
        root: Entity,
        lobby: &LobbySnapshot,
        context: GameLobbyContext<'_>,
    ) {
        render_guess_lobby(
            commands,
            root,
            context.client,
            lobby,
            &self.ui,
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
            assets,
            avatars,
            game_summary,
            transition,
            turn_border_materials,
            ..
        } = context;
        render_guess_table(
            commands,
            root,
            GuessTable {
                client,
                game,
                state: &self.ui,
                graph_art: &self.art,
                scores: &self.scores,
                feedback: &self.feedback,
                summary: game_summary,
                game_assets: &self.assets,
                turn_border_materials,
                social,
                chat,
                assets,
                avatars,
                transition,
            },
        );
    }
}

impl GameActionPrompt for McGuessScreen<'_> {
    fn requires_action(&self, game: &Self::Snapshot) -> bool {
        // 连击需要先选相邻节点时也在等待本人操作，不能只检查当前选中的节点。
        game.round.can_guess(game.you, game.rules.mode)
            && game
                .players
                .iter()
                .any(|player| player.player.id == game.you)
    }
}

impl GameUiLifecycle for McGuessScreen<'_> {
    fn reconcile(&mut self, client: Option<&ClientResource>, _social: &mut SocialUiState) {
        let playing = client
            .and_then(|client| client.0.model().game_snapshot())
            .and_then(GameSnapshot::mc_guess)
            .is_some();
        let in_lobby = client.is_some_and(|client| {
            client
                .0
                .model()
                .lobby()
                .is_some_and(|lobby| lobby.rules.mc_guess().is_some())
        });
        if !playing && !in_lobby {
            self.ui.mode_menu_open = false;
        }
    }
}
