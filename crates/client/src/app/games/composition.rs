//! 游戏大厅与牌桌的运行期组合边界。

use super::mahjong::{
    MahjongAssets, MahjongClaimPresentationState, MahjongTableVisuals, MahjongTileMaterial,
    MahjongUiState, render_mahjong_lobby, render_mahjong_table,
};
use super::qigui523::{
    PlayEffectState, QiGui523Assets, QiGui523UiState, TableVisualContext, render_qigui523_lobby,
    render_table,
};
use super::shengji::{
    ShengjiAssets, ShengjiPresentationState, ShengjiScoreCaptureEffectState,
    ShengjiSettlementAnimation, ShengjiTableVisuals, ShengjiUiState, render_shengji_lobby,
    render_shengji_table,
};
use super::texas_holdem::{
    TexasChipTableState, TexasHoldemAssets, TexasHoldemUiState, TexasTableVisuals,
    render_texas_holdem_lobby, render_texas_holdem_table,
};
use super::uno::{UnoAssets, UnoTableVisuals, UnoUiState, render_uno_lobby, render_uno_table};
use crate::app::presentation::{
    GameSummaryAnimation, StartGameSeatTransition, TableBackgroundMaterial, TurnBorderMaterial,
};
use crate::app::runtime::{
    AppearancePreferences, AvatarImages, ClientResource, TableAppearance, UiAssets,
};
use crate::app::shell::{ChatPanelState, DeveloperHandInput, ScoreCaptureEffectState, UiState};
use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use leocard_client::ClientPhaseRef;
use leocard_protocol::{GameKind, GameSnapshot, LobbySnapshot, PlayerId};

#[derive(SystemParam)]
pub(crate) struct GameScreenResources<'w> {
    qigui523_assets: Res<'w, QiGui523Assets>,
    texas_holdem_assets: Res<'w, TexasHoldemAssets>,
    shengji_assets: Res<'w, ShengjiAssets>,
    uno_assets: Res<'w, UnoAssets>,
    mahjong_assets: Res<'w, MahjongAssets>,
    qigui523_ui: ResMut<'w, QiGui523UiState>,
    texas_holdem_ui: ResMut<'w, TexasHoldemUiState>,
    shengji_ui: ResMut<'w, ShengjiUiState>,
    uno_ui: ResMut<'w, UnoUiState>,
    mahjong_ui: ResMut<'w, MahjongUiState>,
    play_effect: Res<'w, PlayEffectState>,
    score_capture: Res<'w, ScoreCaptureEffectState>,
    shengji_score_capture: Res<'w, ShengjiScoreCaptureEffectState>,
    shengji_settlement: Res<'w, ShengjiSettlementAnimation>,
    shengji_presentation: Res<'w, ShengjiPresentationState>,
    mahjong_claim_presentation: Res<'w, MahjongClaimPresentationState>,
    start_game_transition: Res<'w, StartGameSeatTransition>,
    texas_chips: Res<'w, TexasChipTableState>,
    table_materials: ResMut<'w, Assets<TableBackgroundMaterial>>,
    mahjong_tile_materials: ResMut<'w, Assets<MahjongTileMaterial>>,
    turn_border_materials: ResMut<'w, Assets<TurnBorderMaterial>>,
}

impl GameScreenResources<'_> {
    pub(crate) fn reconcile_screen_state(
        &mut self,
        client: Option<&ClientResource>,
        shell: &mut UiState,
    ) {
        let phase = client
            .map(|client| client.0.model().phase())
            .unwrap_or(ClientPhaseRef::Idle);
        if !matches!(phase, ClientPhaseRef::Lobby(lobby) if lobby.game == GameKind::Uno) {
            self.uno_ui.mode_menu_open = false;
            self.uno_ui.expansion_settings_open = false;
        }
        match phase {
            ClientPhaseRef::Playing(GameSnapshot::QiGui523(game)) => {
                self.qigui523_ui.reconcile(game);
                self.clear_except(GameKind::QiGui523);
                retain_or_clear(shell, |target| {
                    game.players.iter().any(|player| player.id == target)
                });
            }
            ClientPhaseRef::Playing(GameSnapshot::TexasHoldem(game)) => {
                self.texas_holdem_ui.reconcile(game);
                self.clear_except(GameKind::TexasHoldem);
                retain_or_clear(shell, |target| {
                    game.players.iter().any(|player| player.id == target)
                });
            }
            ClientPhaseRef::Playing(GameSnapshot::Shengji(game)) => {
                self.shengji_ui.reconcile(game);
                self.clear_except(GameKind::Shengji);
                retain_or_clear(shell, |target| {
                    game.players.iter().any(|player| player.id == target)
                });
            }
            ClientPhaseRef::Playing(GameSnapshot::Uno(game)) => {
                self.clear_except(GameKind::Uno);
                self.uno_ui.reconcile(game, &mut shell.social);
                retain_or_clear(shell, |target| {
                    game.players.iter().any(|player| player.id == target)
                });
            }
            ClientPhaseRef::Playing(GameSnapshot::Mahjong(game)) => {
                self.clear_except(GameKind::Mahjong);
                retain_or_clear(shell, |target| {
                    game.players.iter().any(|player| player.id == target)
                });
            }
            ClientPhaseRef::Idle | ClientPhaseRef::Lobby(_) | ClientPhaseRef::Closed => {
                self.clear_all();
                shell.social.interaction_menu_open = None;
            }
        }
    }

    fn clear_except(&mut self, active: GameKind) {
        if active != GameKind::QiGui523 {
            self.qigui523_ui.clear();
        }
        if active != GameKind::TexasHoldem {
            self.texas_holdem_ui.clear();
        }
        if active != GameKind::Shengji {
            self.shengji_ui.clear();
        }
        if active != GameKind::Uno {
            self.uno_ui.clear();
        }
        if active != GameKind::Mahjong {
            self.mahjong_ui.clear();
        }
    }

    fn clear_all(&mut self) {
        self.qigui523_ui.clear();
        self.texas_holdem_ui.clear();
        self.shengji_ui.clear();
        self.uno_ui.clear();
        self.mahjong_ui.clear();
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn render_lobby(
        &mut self,
        commands: &mut Commands,
        root: Entity,
        client: &ClientResource,
        lobby: &LobbySnapshot,
        _ui: &UiState,
        assets: &UiAssets,
        avatars: &AvatarImages,
    ) {
        match lobby.game {
            GameKind::QiGui523 => {
                render_qigui523_lobby(commands, root, client, lobby, assets, avatars)
            }
            GameKind::TexasHoldem => {
                render_texas_holdem_lobby(commands, root, client, lobby, assets, avatars)
            }
            GameKind::Shengji => {
                render_shengji_lobby(commands, root, client, lobby, assets, avatars)
            }
            GameKind::Uno => {
                render_uno_lobby(commands, root, client, lobby, &self.uno_ui, assets, avatars)
            }
            GameKind::Mahjong => {
                render_mahjong_lobby(commands, root, client, lobby, assets, avatars)
            }
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn render_table(
        &mut self,
        commands: &mut Commands,
        root: Entity,
        client: &ClientResource,
        game: &GameSnapshot,
        ui: &mut UiState,
        chat: &ChatPanelState,
        developer_hand: &DeveloperHandInput,
        form: &AppearancePreferences,
        assets: &UiAssets,
        avatars: &AvatarImages,
        appearance: &TableAppearance,
        game_summary: &GameSummaryAnimation,
    ) {
        match game {
            GameSnapshot::QiGui523(game) => {
                let mut visuals = TableVisualContext {
                    assets,
                    game_assets: &self.qigui523_assets,
                    avatars,
                    appearance,
                    brightness: form.table_brightness,
                    vignette: form.table_vignette,
                    table_materials: &mut self.table_materials,
                    game_summary,
                    play_effect: &self.play_effect,
                    score_capture: &self.score_capture,
                    start_game_transition: &self.start_game_transition,
                    turn_border_materials: &mut self.turn_border_materials,
                };
                render_table(
                    commands,
                    root,
                    client,
                    game,
                    &self.qigui523_ui,
                    &ui.social,
                    chat,
                    developer_hand,
                    &mut visuals,
                );
            }
            GameSnapshot::TexasHoldem(game) => render_texas_holdem_table(
                commands,
                root,
                client,
                game,
                &mut self.texas_holdem_ui,
                &ui.social,
                chat,
                TexasTableVisuals {
                    assets,
                    game_assets: &self.texas_holdem_assets,
                    avatars,
                    appearance,
                    brightness: form.table_brightness,
                    vignette: form.table_vignette,
                    table_materials: &mut self.table_materials,
                    turn_border_materials: &mut self.turn_border_materials,
                    start_game_transition: &self.start_game_transition,
                    chip_state: &self.texas_chips,
                    game_summary,
                },
            ),
            GameSnapshot::Shengji(game) => render_shengji_table(
                commands,
                root,
                client,
                game,
                &mut self.shengji_ui,
                &ui.social,
                chat,
                ShengjiTableVisuals {
                    assets,
                    game_assets: &self.shengji_assets,
                    avatars,
                    appearance,
                    brightness: form.table_brightness,
                    vignette: form.table_vignette,
                    table_materials: &mut self.table_materials,
                    turn_border_materials: &mut self.turn_border_materials,
                    start_game_transition: &self.start_game_transition,
                    score_capture: &self.shengji_score_capture,
                    settlement: &self.shengji_settlement,
                    presentation: &self.shengji_presentation,
                },
            ),
            GameSnapshot::Uno(game) => render_uno_table(
                commands,
                root,
                client,
                game,
                &self.uno_ui,
                &ui.social,
                chat,
                UnoTableVisuals {
                    assets,
                    game_assets: &self.uno_assets,
                    avatars,
                    appearance,
                    brightness: form.table_brightness,
                    vignette: form.table_vignette,
                    table_materials: &mut self.table_materials,
                    turn_border_materials: &mut self.turn_border_materials,
                    game_summary,
                },
            ),
            GameSnapshot::Mahjong(game) => {
                let interaction_menu_open = ui.social.interaction_menu_open;
                render_mahjong_table(
                    commands,
                    root,
                    client,
                    game,
                    &mut self.mahjong_ui,
                    chat,
                    interaction_menu_open,
                    MahjongTableVisuals {
                        assets,
                        game_assets: &self.mahjong_assets,
                        avatars,
                        developer_hand,
                        appearance,
                        brightness: form.table_brightness,
                        vignette: form.table_vignette,
                        table_materials: &mut self.table_materials,
                        tile_materials: &mut self.mahjong_tile_materials,
                        game_summary,
                        claim_presentation: &self.mahjong_claim_presentation,
                    },
                );
            }
        }
    }
}

fn retain_or_clear(shell: &mut UiState, contains: impl FnOnce(PlayerId) -> bool) {
    if shell
        .social
        .interaction_menu_open
        .is_some_and(|target| !contains(target))
    {
        shell.social.interaction_menu_open = None;
    }
}
