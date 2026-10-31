//! This game owns its screen resources, state reconciliation and view assembly.

use super::super::{GameLobbyContext, GameScreen, GameTableContext};
use super::{
    GraphArt, GuessFeedback, GuessScorePlayback, GuessTable, McGuessAssets, McGuessUiState,
    render_guess_lobby, render_guess_table,
};
use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use leocard_protocol::{LobbySnapshot, McGuessSnapshot};
use leocard_ui::UiComposer;

#[derive(SystemParam)]
pub(crate) struct McGuessScreen<'w> {
    art: Res<'w, GraphArt>,
    assets: Res<'w, McGuessAssets>,
    ui: Res<'w, McGuessUiState>,
    scores: Res<'w, GuessScorePlayback>,
    feedback: Res<'w, GuessFeedback>,
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
