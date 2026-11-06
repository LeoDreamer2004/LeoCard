use super::playback::LinkPlayback;
use super::state::MahjongLinkUiState;
use super::{actions, lobby, table};
use crate::app::games::{
    GameActionPrompt, GameLobbyContext, GameScreen, GameTableContext, GameUiLifecycle,
};
use crate::app::presentation::mahjong::MahjongTileAssets;
use crate::app::runtime::ClientResource;
use crate::app::shell::{SocialUiState, UiActionSet};
use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use leocard_protocol::{GameSnapshot, LobbySnapshot, MahjongLinkSnapshot};
use leocard_ui::UiComposer;

pub(crate) struct MahjongLinkPlugin;

impl Plugin for MahjongLinkPlugin {
    fn build(&self, app: &mut App) {
        super::playback::register(app);
        app.init_resource::<MahjongLinkUiState>()
            .add_systems(Update, actions::dispatch_actions.in_set(UiActionSet));
    }
}

#[derive(SystemParam)]
pub(crate) struct MahjongLinkScreen<'w> {
    ui: ResMut<'w, MahjongLinkUiState>,
    assets: Res<'w, MahjongTileAssets>,
    playback: Res<'w, LinkPlayback>,
}

impl GameScreen for MahjongLinkScreen<'_> {
    type Snapshot = MahjongLinkSnapshot;

    fn render_lobby(
        &self,
        commands: &mut UiComposer,
        root: Entity,
        lobby: &LobbySnapshot,
        context: GameLobbyContext<'_>,
    ) {
        lobby::render(commands, root, lobby, context);
    }

    fn render_table(
        &mut self,
        commands: &mut UiComposer,
        root: Entity,
        game: &Self::Snapshot,
        context: GameTableContext<'_>,
    ) {
        table::render(
            commands,
            root,
            game,
            &self.ui,
            &self.assets,
            &self.playback,
            context,
        );
    }
}

impl GameActionPrompt for MahjongLinkScreen<'_> {
    fn requires_action(&self, game: &Self::Snapshot) -> bool {
        !game.match_complete
            && game.current_player == Some(game.you)
            && game
                .players
                .iter()
                .any(|player| player.player.id == game.you && player.connected && !player.auto_play)
    }
}

impl GameUiLifecycle for MahjongLinkScreen<'_> {
    fn reconcile(&mut self, client: Option<&ClientResource>, _social: &mut SocialUiState) {
        if let Some(game) = client
            .and_then(|client| client.0.model().game_snapshot())
            .and_then(GameSnapshot::mahjong_link)
        {
            self.ui.reconcile(game);
        } else {
            self.ui.clear_game();
        }
    }
}
