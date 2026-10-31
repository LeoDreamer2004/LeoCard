//! This game owns its screen resources, state reconciliation and view assembly.

use super::super::{GameLobbyContext, GameScreen, GameTableContext};
use super::{
    UnoAssets, UnoJumpInDevice, UnoTableVisuals, UnoUiState, render_uno_lobby, render_uno_table,
};
use crate::app::runtime::ClientResource;
use crate::app::shell::SocialUiState;
use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use leocard_client::ClientPhaseRef;
use leocard_protocol::{GameKind, GameSnapshot, LobbySnapshot, UnoSnapshot};
use leocard_ui::UiComposer;

#[derive(SystemParam)]
pub(crate) struct UnoScreen<'w> {
    assets: Res<'w, UnoAssets>,
    ui: ResMut<'w, UnoUiState>,
    jump_in_device: Res<'w, UnoJumpInDevice>,
}

impl GameScreen for UnoScreen<'_> {
    type Snapshot = UnoSnapshot;

    fn requires_action(&self, game: &Self::Snapshot) -> bool {
        use leocard_protocol::{UnoPendingSwapView, UnoPhaseView};
        if !matches!(game.phase, UnoPhaseView::Playing)
            || !game.players.iter().any(|player| {
                player.player.id == game.you
                    && player.connected
                    && !player.auto_play
                    && !player.eliminated
            })
        {
            return false;
        }
        match game.pending_swap {
            Some(
                UnoPendingSwapView::SwapOneTarget { player }
                | UnoPendingSwapView::SwapOneGive { player, .. }
                | UnoPendingSwapView::ForceTrade { player }
                | UnoPendingSwapView::ChooseColor { player }
                | UnoPendingSwapView::SevenSwap { player }
                | UnoPendingSwapView::ColorRoulette { player },
            ) => player == game.you,
            // 抢出、喊 UNO 和检举都是可选动作，不能据此通知回合外的玩家。
            None => game.current_player == Some(game.you),
        }
    }

    fn reconcile(&mut self, client: Option<&ClientResource>, social: &mut SocialUiState) {
        let phase = client
            .map(|client| client.0.model().phase())
            .unwrap_or(ClientPhaseRef::Idle);
        if !matches!(phase, ClientPhaseRef::Lobby(lobby) if lobby.game == GameKind::Uno) {
            self.ui.mode_menu_open = false;
            self.ui.expansion_settings_open = false;
            self.ui.expansion_settings_progress = 0.0;
        }
        match phase {
            ClientPhaseRef::Playing(GameSnapshot::Uno(game)) => self.ui.reconcile(game, social),
            ClientPhaseRef::Playing(_) => self.ui.clear(),
            _ => self.ui.clear_table(),
        }
    }

    fn render_lobby(
        &self,
        commands: &mut UiComposer,
        root: Entity,
        lobby: &LobbySnapshot,
        context: GameLobbyContext<'_>,
    ) {
        render_uno_lobby(
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
        render_uno_table(
            commands,
            root,
            client,
            game,
            &self.ui,
            social,
            chat,
            UnoTableVisuals {
                assets,
                game_assets: &self.assets,
                avatars,
                appearance,
                brightness: form.table_brightness,
                vignette: form.table_vignette,
                table_materials,
                turn_border_materials,
                start_game_transition: transition,
                game_summary,
                jump_in_device: &self.jump_in_device,
            },
        );
    }
}
