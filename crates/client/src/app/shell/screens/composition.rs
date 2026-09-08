//! 根据网络模型与页面状态装配当前的顶层界面。

use super::*;
use bevy::ecs::system::SystemParam;
use leocard_protocol::GameSnapshot;

#[derive(SystemParam)]
struct VisualAssets<'w> {
    ui: Res<'w, UiAssets>,
    avatars: Res<'w, AvatarImages>,
    table: Res<'w, TableAppearance>,
    play_error: Res<'w, PlayErrorToast>,
    game_summary: Res<'w, GameSummaryAnimation>,
    play_effect: Res<'w, PlayEffectState>,
    score_capture: Res<'w, ScoreCaptureEffectState>,
    shengji_score_capture: Res<'w, ShengjiScoreCaptureEffectState>,
    shengji_settlement: Res<'w, ShengjiSettlementAnimation>,
    shengji_presentation: Res<'w, ShengjiPresentationState>,
    mahjong_claim_presentation: Res<'w, MahjongClaimPresentationState>,
    start_game_transition: Res<'w, StartGameSeatTransition>,
    texas_chips: Res<'w, TexasChipTableState>,
    updater: Res<'w, UpdateManager>,
}

#[derive(SystemParam)]
struct ScreenResources<'w> {
    client: Option<Res<'w, ClientResource>>,
    form: Res<'w, ConnectionForm>,
    profile: Res<'w, LocalPlayerProfile>,
    chat: Res<'w, ChatPanelState>,
    developer_hand: Res<'w, DeveloperHandInput>,
    ui: ResMut<'w, UiState>,
}

#[derive(SystemParam)]
struct ScreenMaterials<'w> {
    table: ResMut<'w, Assets<TableBackgroundMaterial>>,
    mahjong_tiles: ResMut<'w, Assets<MahjongTileMaterial>>,
    turn_borders: ResMut<'w, Assets<TurnBorderMaterial>>,
}

#[derive(SystemParam)]
pub struct ScreenRebuild<'w, 's> {
    commands: Commands<'w, 's>,
    resources: ScreenResources<'w>,
    visuals: VisualAssets<'w>,
    materials: ScreenMaterials<'w>,
    roots: Query<'w, 's, Entity, With<UiRoot>>,
}

pub fn rebuild_ui(mut screen: ScreenRebuild) {
    screen.rebuild();
}

impl ScreenRebuild<'_, '_> {
    fn rebuild(&mut self) {
        if !self.resources.ui.dirty {
            return;
        }
        self.resources.ui.dirty = false;
        self.resources
            .ui
            .reconcile_screen_state(self.resources.client.as_deref());
        for entity in &self.roots {
            self.commands.entity(entity).despawn();
        }

        let root = self
            .commands
            .spawn((
                UiRoot,
                Node {
                    width: percent(100),
                    height: percent(100),
                    flex_direction: FlexDirection::Column,
                    ..default()
                },
            ))
            .id();
        ScreenRenderer {
            commands: &mut self.commands,
            client: self.resources.client.as_deref(),
            form: &self.resources.form,
            profile: &self.resources.profile,
            chat: &self.resources.chat,
            developer_hand: &self.resources.developer_hand,
            ui: &mut self.resources.ui,
            visuals: &self.visuals,
            materials: &mut self.materials,
        }
        .render(root);
    }
}

struct ScreenRenderer<'a, 'w, 's> {
    commands: &'a mut Commands<'w, 's>,
    client: Option<&'a ClientResource>,
    form: &'a ConnectionForm,
    profile: &'a LocalPlayerProfile,
    chat: &'a ChatPanelState,
    developer_hand: &'a DeveloperHandInput,
    ui: &'a mut UiState,
    visuals: &'a VisualAssets<'w>,
    materials: &'a mut ScreenMaterials<'w>,
}

impl ScreenRenderer<'_, '_, '_> {
    fn render(&mut self, root: Entity) {
        Header::new(
            self.client,
            self.form,
            &self.visuals.ui,
            &self.visuals.avatars,
        )
        .render(self.commands, root);
        self.render_primary_screen(root);
        self.render_overlays(root);
    }

    fn render_primary_screen(&mut self, root: Entity) {
        let Some(client) = self.client else {
            ConnectionScreen::new(self.form, None, &self.visuals.ui, &self.visuals.avatars)
                .render(self.commands, root);
            return;
        };
        let model = client.0.model();
        if let Some(lobby) = model.lobby() {
            LobbyScreen::new(
                client,
                lobby,
                self.ui,
                &self.visuals.ui,
                &self.visuals.avatars,
            )
            .render(self.commands, root);
        } else if let Some(game) = model.game_snapshot() {
            self.render_game(root, client, game);
        } else {
            ConnectionScreen::new(
                self.form,
                Some(&client.0),
                &self.visuals.ui,
                &self.visuals.avatars,
            )
            .render(self.commands, root);
        }
    }

    fn render_game(&mut self, root: Entity, client: &ClientResource, game: &GameSnapshot) {
        match game {
            GameSnapshot::QiGui523(game) => {
                let mut visuals = TableVisualContext {
                    assets: &self.visuals.ui,
                    avatars: &self.visuals.avatars,
                    appearance: &self.visuals.table,
                    brightness: self.form.table_brightness,
                    vignette: self.form.table_vignette,
                    table_materials: &mut self.materials.table,
                    game_summary: &self.visuals.game_summary,
                    play_effect: &self.visuals.play_effect,
                    score_capture: &self.visuals.score_capture,
                    start_game_transition: &self.visuals.start_game_transition,
                    turn_border_materials: &mut self.materials.turn_borders,
                };
                render_table(
                    self.commands,
                    root,
                    client,
                    game,
                    self.ui,
                    self.chat,
                    self.developer_hand,
                    &mut visuals,
                );
            }
            GameSnapshot::TexasHoldem(game) => render_texas_holdem_table(
                self.commands,
                root,
                client,
                game,
                self.ui,
                self.chat,
                TexasTableVisuals {
                    assets: &self.visuals.ui,
                    avatars: &self.visuals.avatars,
                    appearance: &self.visuals.table,
                    brightness: self.form.table_brightness,
                    vignette: self.form.table_vignette,
                    table_materials: &mut self.materials.table,
                    turn_border_materials: &mut self.materials.turn_borders,
                    start_game_transition: &self.visuals.start_game_transition,
                    chip_state: &self.visuals.texas_chips,
                    game_summary: &self.visuals.game_summary,
                },
            ),
            GameSnapshot::Shengji(game) => render_shengji_table(
                self.commands,
                root,
                client,
                game,
                self.ui,
                self.chat,
                ShengjiTableVisuals {
                    assets: &self.visuals.ui,
                    avatars: &self.visuals.avatars,
                    appearance: &self.visuals.table,
                    brightness: self.form.table_brightness,
                    vignette: self.form.table_vignette,
                    table_materials: &mut self.materials.table,
                    turn_border_materials: &mut self.materials.turn_borders,
                    start_game_transition: &self.visuals.start_game_transition,
                    score_capture: &self.visuals.shengji_score_capture,
                    settlement: &self.visuals.shengji_settlement,
                    presentation: &self.visuals.shengji_presentation,
                },
            ),
            GameSnapshot::Uno(game) => render_uno_table(
                self.commands,
                root,
                client,
                game,
                self.ui,
                self.chat,
                UnoTableVisuals {
                    assets: &self.visuals.ui,
                    avatars: &self.visuals.avatars,
                    appearance: &self.visuals.table,
                    brightness: self.form.table_brightness,
                    vignette: self.form.table_vignette,
                    table_materials: &mut self.materials.table,
                    turn_border_materials: &mut self.materials.turn_borders,
                    game_summary: &self.visuals.game_summary,
                },
            ),
            GameSnapshot::Mahjong(game) => {
                let interaction_menu_open = self.ui.social.interaction_menu_open;
                render_mahjong_table(
                    self.commands,
                    root,
                    client,
                    game,
                    self.ui,
                    self.chat,
                    interaction_menu_open,
                    MahjongTableVisuals {
                        assets: &self.visuals.ui,
                        avatars: &self.visuals.avatars,
                        developer_hand: self.developer_hand,
                        appearance: &self.visuals.table,
                        brightness: self.form.table_brightness,
                        vignette: self.form.table_vignette,
                        table_materials: &mut self.materials.table,
                        tile_materials: &mut self.materials.mahjong_tiles,
                        game_summary: &self.visuals.game_summary,
                        claim_presentation: &self.visuals.mahjong_claim_presentation,
                    },
                );
            }
        }
    }

    fn render_overlays(&mut self, root: Entity) {
        if self.ui.navigation.settings_open {
            SettingsModal::new(self.form, &self.visuals.updater, &self.visuals.ui)
                .render(self.commands, root);
        }
        if self.ui.navigation.profile_open {
            self.render_profile(root);
        }
        if self.ui.navigation.host_game_picker_open && self.client.is_none() {
            HostGamePicker::new(&self.visuals.ui).render(self.commands, root);
        }
        if self.visuals.updater.dialog_open {
            render_update_dialog(self.commands, root, &self.visuals.updater, &self.visuals.ui);
        }
        if self.visuals.play_error.active
            && let Some(message) = self.visuals.play_error.message.as_deref()
        {
            add_play_error_popup(
                self.commands,
                root,
                message,
                &self.visuals.play_error,
                &self.visuals.ui,
            );
        }
    }

    fn render_profile(&mut self, root: Entity) {
        let (name, avatar, reference_points, completed_games, game_profiles) =
            if let Some(player) = self.ui.navigation.player_profile.as_ref() {
                (
                    player.name.as_str(),
                    player.avatar.as_ref(),
                    player.reference_points,
                    player.completed_games,
                    &player.game_profiles,
                )
            } else {
                (
                    self.form.player_name.as_str(),
                    self.visuals.avatars.local.as_ref(),
                    self.profile.reference_points(),
                    self.profile.completed_games(),
                    self.profile.game_profiles(),
                )
            };
        ProfileModal::new(
            name,
            avatar,
            reference_points,
            completed_games,
            game_profiles,
            self.ui.navigation.profile_game_tab,
            &self.visuals.ui,
        )
        .render(self.commands, root);
    }
}
