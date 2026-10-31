//! 斗地主的界面资源借用、状态整理与装配入口。

use super::super::{GameLobbyContext, GameScreen, GameTableContext};
use super::{DoudizhuTable, DoudizhuUiState, render_doudizhu_lobby, render_doudizhu_table};
use crate::app::presentation::PlayEffectState;
use crate::app::runtime::ClientResource;
use crate::app::shell::SocialUiState;
use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use leocard_client::ClientPhaseRef;
use leocard_protocol::{DoudizhuSnapshot, GameKind, GameSnapshot, LobbySnapshot};
use leocard_ui::UiComposer;

#[derive(SystemParam)]
pub(crate) struct DoudizhuScreen<'w> {
    ui: ResMut<'w, DoudizhuUiState>,
    play_effect: Res<'w, PlayEffectState>,
}

impl GameScreen for DoudizhuScreen<'_> {
    type Snapshot = DoudizhuSnapshot;

    fn requires_action(&self, game: &Self::Snapshot) -> bool {
        use leocard_doudizhu::DoudizhuPhase;
        matches!(game.phase, DoudizhuPhase::Bidding | DoudizhuPhase::Playing)
            && game.current == game.you
            && game
                .players
                .iter()
                .any(|player| player.player.id == game.you && player.connected && !player.auto_play)
    }

    fn reconcile(&mut self, client: Option<&ClientResource>, _social: &mut SocialUiState) {
        let phase = client.map(|client| client.0.model().phase());
        if !matches!(phase, Some(ClientPhaseRef::Lobby(lobby)) if lobby.game == GameKind::Doudizhu)
        {
            self.ui.mode_menu_open = false;
        }
        match phase {
            Some(ClientPhaseRef::Playing(GameSnapshot::Doudizhu(game))) => self.ui.reconcile(game),
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
        render_doudizhu_lobby(
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
        render_doudizhu_table(
            commands,
            root,
            DoudizhuTable {
                client: context.client,
                game,
                ui: &mut self.ui,
                social: context.social,
                chat: context.chat,
                assets: context.assets,
                avatars: context.avatars,
                appearance: context.appearance,
                brightness: context.form.table_brightness,
                vignette: context.form.table_vignette,
                table_materials: context.table_materials,
                turn_materials: context.turn_border_materials,
                summary: context.game_summary,
                transition: context.transition,
                play_effect: &self.play_effect,
            },
        );
    }
}
