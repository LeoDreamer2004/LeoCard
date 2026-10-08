//! 根据网络模型与页面状态装配当前的顶层界面。

use super::{
    AchievementPageViewport, AchievementsPage, ChatPanelState, ConfirmationDialog,
    ConnectionScreen, DeveloperHandInput, Header, LobbyGameMotion, PageMotion, PlayErrorToast,
    ProfileModal, ProfileMotion, SettingsModal, SettingsMotion, ShopPage, UiState, UpdateManager,
    add_lobby_game_shade, add_page_background, add_page_transition_shade, add_play_error_popup,
    render_confirmation, render_update_dialog,
};
use crate::app::games::{GameScreenResources, GameScreenRetainedState};
use crate::app::presentation::{GameSummaryAnimation, UiRoot};
use crate::app::runtime::{
    AppearancePreferences, AvatarImages, ClientResource, ConnectionDraft, TableAppearance, UiAssets,
};
use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use leocard_client::{ClientPhaseRef, LocalPlayerProfile, PlayerAchievements, PlayerEconomy};

#[derive(SystemParam)]
struct VisualAssets<'w> {
    ui: Res<'w, UiAssets>,
    avatars: Res<'w, AvatarImages>,
    table: Res<'w, TableAppearance>,
    play_error: Res<'w, PlayErrorToast>,
    game_summary: Res<'w, GameSummaryAnimation>,
    updater: Res<'w, UpdateManager>,
    confirmation: Res<'w, ConfirmationDialog>,
    settings_motion: Res<'w, SettingsMotion>,
    profile_motion: Res<'w, ProfileMotion>,
    page_motion: Res<'w, PageMotion>,
    lobby_game_motion: Res<'w, LobbyGameMotion>,
}

#[derive(SystemParam)]
struct ScreenResources<'w> {
    client: Option<Res<'w, ClientResource>>,
    connection: Res<'w, ConnectionDraft>,
    appearance: Res<'w, AppearancePreferences>,
    profile: Res<'w, LocalPlayerProfile>,
    achievements: Res<'w, PlayerAchievements>,
    economy: Res<'w, PlayerEconomy>,
    chat: Res<'w, ChatPanelState>,
    developer_hand: Res<'w, DeveloperHandInput>,
    ui: ResMut<'w, UiState>,
}

#[derive(Component)]
struct GameIntroRoot;

#[derive(SystemParam)]
pub(crate) struct ScreenRebuild<'w, 's> {
    commands: Commands<'w, 's>,
    resources: ScreenResources<'w>,
    visuals: VisualAssets<'w>,
    games: GameScreenResources<'w>,
    roots: Query<'w, 's, Entity, With<UiRoot>>,
    intro_roots: Query<'w, 's, Entity, With<GameIntroRoot>>,
    achievement_viewport: AchievementPageViewport<'w, 's>,
    retained_game: GameScreenRetainedState<'w, 's>,
}

pub(crate) fn rebuild_ui(mut screen: ScreenRebuild) {
    screen.rebuild();
}

impl ScreenRebuild<'_, '_> {
    fn rebuild(&mut self) {
        if !self.resources.ui.dirty {
            return;
        }
        if self.visuals.lobby_game_motion.hold_lobby() || self.visuals.page_motion.hold_page() {
            return;
        }
        // The intro contains portraits and felt only. Keep its targets alive
        // through opening snapshots, then build the game once it completes.
        let intro_active = self.games.start_transition_active();
        if intro_active && !self.intro_roots.is_empty() {
            return;
        }
        self.resources.ui.dirty = false;
        self.retained_game
            .capture(self.resources.client.as_deref(), &mut self.games);
        self.games
            .reconcile_screen_state(self.resources.client.as_deref(), &mut self.resources.ui);
        // Preserve the actual viewport before replacing the page's entities.
        let achievement_scroll_y = self.achievement_viewport.offset();
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
            achievements: &self.resources.achievements,
            economy: &self.resources.economy,
            chat: &self.resources.chat,
            developer_hand: &self.resources.developer_hand,
            ui: &mut self.resources.ui,
            achievement_scroll_y,
            visuals: &self.visuals,
            games: &mut self.games,
        }
        .render(root);
        if intro_active {
            self.commands.entity(root).insert(GameIntroRoot);
        }
    }
}

struct ScreenRenderer<'a, 'w, 's> {
    commands: &'a mut Commands<'w, 's>,
    client: Option<&'a ClientResource>,
    connection: &'a ConnectionDraft,
    appearance: &'a AppearancePreferences,
    profile: &'a LocalPlayerProfile,
    achievements: &'a PlayerAchievements,
    economy: &'a PlayerEconomy,
    chat: &'a ChatPanelState,
    developer_hand: &'a DeveloperHandInput,
    ui: &'a mut UiState,
    achievement_scroll_y: f32,
    visuals: &'a VisualAssets<'w>,
    games: &'a mut GameScreenResources<'w>,
}

impl ScreenRenderer<'_, '_, '_> {
    fn render(&mut self, root: Entity) {
        if self.ui.shop.open
            || self.ui.achievements.open
            || self.client.is_none_or(|client| {
                !matches!(client.0.model().phase(), ClientPhaseRef::Playing(_))
            })
        {
            add_page_background(self.commands, root);
        }
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
        if self.ui.shop.open {
            ShopPage {
                assets: &self.visuals.ui,
                economy: self.economy,
                state: &self.ui.shop,
                in_game: self.client.is_some_and(|client| {
                    matches!(client.0.model().phase(), ClientPhaseRef::Playing(_))
                }),
            }
            .render(self.commands, root);
            return;
        }
        if self.ui.achievements.open {
            AchievementsPage::new(
                self.ui.achievements.category,
                self.achievement_scroll_y,
                self.achievements,
                &self.visuals.ui,
            )
            .render(self.commands, root);
            return;
        }
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
        render_confirmation(
            self.commands,
            root,
            &self.visuals.confirmation,
            &self.visuals.ui,
        );
        add_page_transition_shade(self.commands, root, &self.visuals.page_motion);
        if self.visuals.lobby_game_motion.active() {
            add_lobby_game_shade(self.commands, root, &self.visuals.lobby_game_motion);
        }
        if self.ui.settings.open {
            SettingsModal::new(self.appearance, &self.visuals.updater, &self.visuals.ui).render(
                self.commands,
                root,
                self.visuals.settings_motion.progress,
                self.ui.settings.tab,
            );
        }
        if self.ui.profile.open {
            self.render_profile(root);
        }
        if self.visuals.updater.dialog_open || self.visuals.updater.dialog_progress > 0.0 {
            render_update_dialog(
                self.commands,
                root,
                &self.visuals.updater,
                self.visuals.updater.dialog_progress,
                &self.visuals.ui,
            );
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
        ProfileModal::for_selection(
            &self.ui.profile,
            self.client,
            self.connection,
            &self.visuals.avatars,
            self.profile,
            &self.visuals.ui,
        )
        .render(self.commands, root, self.visuals.profile_motion.progress);
    }
}
