//! 根据网络模型与页面状态装配当前的顶层界面。

use super::{
    ChatPanelState, ConnectionScreen, DeveloperHandInput, Header, HostGamePicker, PlayErrorToast,
    ProfileModal, SettingsModal, UiState, UpdateManager, add_play_error_popup,
    render_update_dialog,
};
use crate::app::games::GameScreenResources;
use crate::app::presentation::{GameSummaryAnimation, UiRoot};
use crate::app::runtime::{
    AppearancePreferences, AvatarImages, ClientResource, ConnectionDraft, TableAppearance, UiAssets,
};
use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use leocard_client::{ClientPhaseRef, LocalPlayerProfile};

#[derive(SystemParam)]
struct VisualAssets<'w> {
    ui: Res<'w, UiAssets>,
    avatars: Res<'w, AvatarImages>,
    table: Res<'w, TableAppearance>,
    play_error: Res<'w, PlayErrorToast>,
    game_summary: Res<'w, GameSummaryAnimation>,
    updater: Res<'w, UpdateManager>,
}

#[derive(SystemParam)]
struct ScreenResources<'w> {
    client: Option<Res<'w, ClientResource>>,
    connection: Res<'w, ConnectionDraft>,
    appearance: Res<'w, AppearancePreferences>,
    profile: Res<'w, LocalPlayerProfile>,
    chat: Res<'w, ChatPanelState>,
    developer_hand: Res<'w, DeveloperHandInput>,
    ui: ResMut<'w, UiState>,
}

#[derive(SystemParam)]
pub(crate) struct ScreenRebuild<'w, 's> {
    commands: Commands<'w, 's>,
    resources: ScreenResources<'w>,
    visuals: VisualAssets<'w>,
    games: GameScreenResources<'w>,
    roots: Query<'w, 's, Entity, With<UiRoot>>,
}

pub(crate) fn rebuild_ui(mut screen: ScreenRebuild) {
    screen.rebuild();
}

impl ScreenRebuild<'_, '_> {
    fn rebuild(&mut self) {
        if !self.resources.ui.dirty {
            return;
        }
        self.resources.ui.dirty = false;
        self.games
            .reconcile_screen_state(self.resources.client.as_deref(), &mut self.resources.ui);
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
            connection: &self.resources.connection,
            appearance: &self.resources.appearance,
            profile: &self.resources.profile,
            chat: &self.resources.chat,
            developer_hand: &self.resources.developer_hand,
            ui: &mut self.resources.ui,
            visuals: &self.visuals,
            games: &mut self.games,
        }
        .render(root);
    }
}

struct ScreenRenderer<'a, 'w, 's> {
    commands: &'a mut Commands<'w, 's>,
    client: Option<&'a ClientResource>,
    connection: &'a ConnectionDraft,
    appearance: &'a AppearancePreferences,
    profile: &'a LocalPlayerProfile,
    chat: &'a ChatPanelState,
    developer_hand: &'a DeveloperHandInput,
    ui: &'a mut UiState,
    visuals: &'a VisualAssets<'w>,
    games: &'a mut GameScreenResources<'w>,
}

impl ScreenRenderer<'_, '_, '_> {
    fn render(&mut self, root: Entity) {
        Header::new(
            self.client,
            self.connection,
            &self.visuals.ui,
            &self.visuals.avatars,
        )
        .render(self.commands, root);
        self.render_primary_screen(root);
        self.render_overlays(root);
    }

    fn render_primary_screen(&mut self, root: Entity) {
        let Some(client) = self.client else {
            ConnectionScreen::new(
                self.connection,
                self.appearance,
                None,
                &self.visuals.ui,
                &self.visuals.avatars,
            )
            .render(self.commands, root);
            return;
        };
        match client.0.model().phase() {
            ClientPhaseRef::Lobby(lobby) => self.games.render_lobby(
                self.commands,
                root,
                client,
                lobby,
                self.ui,
                &self.visuals.ui,
                &self.visuals.avatars,
            ),
            ClientPhaseRef::Playing(game) => self.games.render_table(
                self.commands,
                root,
                client,
                game,
                self.ui,
                self.chat,
                self.developer_hand,
                self.appearance,
                &self.visuals.ui,
                &self.visuals.avatars,
                &self.visuals.table,
                &self.visuals.game_summary,
            ),
            ClientPhaseRef::Idle | ClientPhaseRef::Closed => ConnectionScreen::new(
                self.connection,
                self.appearance,
                Some(&client.0),
                &self.visuals.ui,
                &self.visuals.avatars,
            )
            .render(self.commands, root),
        }
    }

    fn render_overlays(&mut self, root: Entity) {
        if self.ui.navigation.settings_open {
            SettingsModal::new(self.appearance, &self.visuals.updater, &self.visuals.ui)
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
                    self.connection.player_name.as_str(),
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
