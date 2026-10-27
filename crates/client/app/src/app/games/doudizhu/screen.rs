//! 斗地主的界面资源借用、状态整理与装配入口。

use super::super::{GameLobbyContext, GameScreen, GameTableContext};
use super::{DoudizhuTable, DoudizhuUiState, render_doudizhu_lobby, render_doudizhu_table};
use crate::app::runtime::ClientResource;
use crate::app::shell::SocialUiState;
use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use leocard_client::ClientPhaseRef;
use leocard_protocol::{DoudizhuSnapshot, GameSnapshot, LobbySnapshot};
use leocard_ui::UiComposer;

#[derive(SystemParam)]
pub(crate) struct DoudizhuScreen<'w> {
    ui: ResMut<'w, DoudizhuUiState>,
}

impl GameScreen for DoudizhuScreen<'_> {
    type Snapshot = DoudizhuSnapshot;

    fn reconcile(&mut self, client: Option<&ClientResource>, _social: &mut SocialUiState) {
        match client.map(|client| client.0.model().phase()) {
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
            },
        );
    }
}
