//! This game owns its screen resources, state reconciliation and view assembly.
use super::super::{GameLobbyContext, GameScreen, GameTableContext};
use super::{
    ShengjiAssets, ShengjiClientState, ShengjiPresentationState, ShengjiScoreCaptureEffectState,
    ShengjiSettlementAnimation, ShengjiTableVisuals, ShengjiUiState, render_shengji_lobby,
    render_shengji_table,
};
use crate::app::runtime::ClientResource;
use crate::app::shell::SocialUiState;
use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use leocard_client::ClientPhaseRef;
use leocard_protocol::{GameKind, GameSnapshot, LobbySnapshot, ShengjiSnapshot};
use leocard_ui::UiComposer;

#[derive(SystemParam)]
pub(crate) struct ShengjiScreen<'w> {
    client_state: Res<'w, ShengjiClientState>,
    assets: Res<'w, ShengjiAssets>,
    ui: ResMut<'w, ShengjiUiState>,
    score_capture: Res<'w, ShengjiScoreCaptureEffectState>,
    settlement: Res<'w, ShengjiSettlementAnimation>,
    presentation: Res<'w, ShengjiPresentationState>,
}

impl GameScreen for ShengjiScreen<'_> {
    type Snapshot = ShengjiSnapshot;

    fn requires_action(&self, game: &Self::Snapshot) -> bool {
        use super::view::shengji_declaration_candidate;
        use leocard_game_common::PlayerId;
        use leocard_protocol::ShengjiPhaseView;
        use leocard_shengji::{ShengjiFiveTrumpCrossingStage, ShengjiSuit};
        if !game
            .players
            .iter()
            .any(|player| player.player.id == game.you && player.connected && !player.auto_play)
        {
            return false;
        }
        match &game.phase {
            ShengjiPhaseView::Dealing { .. } | ShengjiPhaseView::BiddingGrace { .. } => {
                matches!(
                    game.phase,
                    ShengjiPhaseView::BiddingGrace {
                        you_confirmed: false,
                        ..
                    }
                ) || [
                    Some(ShengjiSuit::Spade),
                    Some(ShengjiSuit::Heart),
                    Some(ShengjiSuit::Club),
                    Some(ShengjiSuit::Diamond),
                    None,
                ]
                .into_iter()
                .any(|suit| shengji_declaration_candidate(game, suit).is_some())
            }
            ShengjiPhaseView::Burying => game.dealer == Some(game.you),
            ShengjiPhaseView::BottomCopying { player, .. }
            | ShengjiPhaseView::BottomCopyBurying { player } => *player == game.you,
            ShengjiPhaseView::FiveTrumpCrossing {
                stage: ShengjiFiveTrumpCrossingStage::Deciding,
                eligible,
                decided,
                ..
            } => eligible.contains(&game.you) && !decided.contains(&game.you),
            ShengjiPhaseView::FiveTrumpCrossing {
                stage: ShengjiFiveTrumpCrossingStage::Returning,
                crossing,
                returned,
                ..
            } => {
                crossing.contains(&PlayerId((game.you.0 + 2) % 4)) && !returned.contains(&game.you)
            }
            ShengjiPhaseView::Playing => {
                game.current_player == Some(game.you) && game.throw_failure.is_none()
            }
            ShengjiPhaseView::BottomFlipping { .. }
            | ShengjiPhaseView::Finished { .. }
            | ShengjiPhaseView::Redealing => false,
        }
    }

    fn reconcile(&mut self, client: Option<&ClientResource>, _social: &mut SocialUiState) {
        let phase = client.map(|client| client.0.model().phase());
        if !matches!(phase, Some(ClientPhaseRef::Lobby(lobby)) if lobby.game == GameKind::Shengji) {
            self.ui.mode_menu_open = false;
        }
        match phase {
            Some(ClientPhaseRef::Playing(GameSnapshot::Shengji(game))) => self.ui.reconcile(game),
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
        render_shengji_lobby(
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
            transition,
            table_materials,
            turn_border_materials,
            ..
        } = context;
        render_shengji_table(
            commands,
            root,
            client,
            game,
            &mut self.ui,
            social,
            chat,
            ShengjiTableVisuals {
                client_state: &self.client_state,
                assets,
                game_assets: &self.assets,
                avatars,
                appearance,
                brightness: form.table_brightness,
                vignette: form.table_vignette,
                table_materials,
                turn_border_materials,
                start_game_transition: transition,
                score_capture: &self.score_capture,
                settlement: &self.settlement,
                presentation: &self.presentation,
            },
        );
    }
}
