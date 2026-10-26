//! 根据网络模型与页面状态装配当前的顶层界面。

use super::connection::{LanDiscovery, render_lan_rooms};
use super::{
    AchievementPageViewport, AchievementsPage, ChatPanelState, ConfirmationDialog,
    ConnectionScreen, DeveloperHandInput, Header, LobbyGameMotion, PageMotion, PlayErrorToast,
    ProfileModal, ProfileMotion, SettingsModal, SettingsMotion, ShopPage, UiState, UpdateManager,
    add_lobby_game_shade, add_page_background, add_page_transition_shade, add_play_error_popup,
    render_confirmation, render_update_dialog,
};
#[cfg(not(target_os = "android"))]
use super::{SakuraPage, SakuraState, render_startup_dialog};
use crate::app::games::{GameLobbyContext, GameScreens, GameTableContext};
use crate::app::presentation::{
    GameSummaryAnimation, StartGameSeatTransition, TableBackgroundMaterial, TurnBorderMaterial,
    UiRoot,
};
use crate::app::runtime::{
    AppearancePreferences, AvatarImages, ClientResource, ConnectionDraft, TableAppearance, UiAssets,
};
use bevy::ecs::system::SystemParam;
use bevy::input_focus::tab_navigation::TabGroup;
use bevy::prelude::*;
use leocard_client::{ClientPhaseRef, LocalPlayerProfile, PlayerAchievements, PlayerEconomy};
use leocard_ui::UiComposer;

#[derive(SystemParam)]
struct VisualAssets<'w> {
    ui: Res<'w, UiAssets>,
    avatars: Res<'w, AvatarImages>,
    lan: Res<'w, LanDiscovery>,
    table: Res<'w, TableAppearance>,
    play_error: Res<'w, PlayErrorToast>,
    game_summary: Res<'w, GameSummaryAnimation>,
    start_game_transition: Res<'w, StartGameSeatTransition>,
    table_materials: ResMut<'w, Assets<TableBackgroundMaterial>>,
    turn_border_materials: ResMut<'w, Assets<TurnBorderMaterial>>,
    updater: Res<'w, UpdateManager>,
    confirmation: Res<'w, ConfirmationDialog>,
    settings_motion: Res<'w, SettingsMotion>,
    profile_motion: Res<'w, ProfileMotion>,
    page_motion: Res<'w, PageMotion>,
    lobby_game_motion: Res<'w, LobbyGameMotion>,
    #[cfg(not(target_os = "android"))]
    sakura: Res<'w, SakuraState>,
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
pub(crate) struct ScreenComposition<'w, 's> {
    commands: UiComposer<'w, 's>,
    resources: ScreenResources<'w>,
    visuals: VisualAssets<'w>,
    games: GameScreens<'w>,
    roots: Query<'w, 's, Entity, With<UiRoot>>,
    intro_roots: Query<'w, 's, Entity, With<GameIntroRoot>>,
    achievement_viewport: AchievementPageViewport<'w, 's>,
}

pub(crate) fn compose_ui(mut screen: ScreenComposition) {
    screen.compose();
}

impl ScreenComposition<'_, '_> {
    fn compose(&mut self) {
        if !self.resources.ui.dirty {
            return;
        }
        if self.visuals.lobby_game_motion.hold_lobby() || self.visuals.page_motion.hold_page() {
            return;
        }
        // The intro contains portraits and felt only. Keep its targets alive
        // through opening snapshots, then build the game once it completes.
        let intro_active = self.visuals.start_game_transition.match_id.is_some()
            && !self.visuals.start_game_transition.seats.is_empty();
        if intro_active && !self.intro_roots.is_empty() {
            return;
        }
        self.resources.ui.dirty = false;
        self.games
            .reconcile_screen_state(self.resources.client.as_deref(), &mut self.resources.ui);
        // Preserve the viewport across category and page transitions.
        let achievement_scroll_y = self.achievement_viewport.offset();
        let root = self.roots.iter().next().unwrap_or_else(|| {
            self.commands
                .spawn((
                    UiRoot,
                    TabGroup::default(),
                    Node {
                        width: percent(100),
                        height: percent(100),
                        flex_direction: FlexDirection::Column,
                        ..default()
                    },
                ))
                .id()
        });
        self.commands.begin(root);
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
            visuals: &mut self.visuals,
            games: &mut self.games,
        }
        .render(root);
        if intro_active {
            self.commands.entity(root).insert(GameIntroRoot);
        } else {
            self.commands.entity(root).remove::<GameIntroRoot>();
        }
        self.commands.finish();
    }
}

struct ScreenRenderer<'a, 'w, 's> {
    commands: &'a mut UiComposer<'w, 's>,
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
    visuals: &'a mut VisualAssets<'w>,
    games: &'a mut GameScreens<'w>,
}

impl ScreenRenderer<'_, '_, '_> {
    fn render(&mut self, root: Entity) {
        if self.ui.shop.open
            || self.sakura_open()
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
        #[cfg(not(target_os = "android"))]
        if self.visuals.sakura.open {
            SakuraPage {
                state: &self.visuals.sakura,
                assets: &self.visuals.ui,
            }
            .render(self.commands, root);
            return;
        }
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
                lobby,
                GameLobbyContext {
                    client,
                    assets: &self.visuals.ui,
                    avatars: &self.visuals.avatars,
                },
            ),
            ClientPhaseRef::Playing(game) => self.games.render_table(
                self.commands,
                root,
                game,
                GameTableContext {
                    client,
                    social: &self.ui.social,
                    chat: self.chat,
                    developer_hand: self.developer_hand,
                    form: self.appearance,
                    assets: &self.visuals.ui,
                    avatars: &self.visuals.avatars,
                    appearance: &self.visuals.table,
                    game_summary: &self.visuals.game_summary,
                    transition: &self.visuals.start_game_transition,
                    table_materials: &mut self.visuals.table_materials,
                    turn_border_materials: &mut self.visuals.turn_border_materials,
                },
            ),
            ClientPhaseRef::Idle | ClientPhaseRef::Closed => ConnectionScreen::new(
                self.connection,
                self.appearance,
                &self.visuals.ui,
                &self.visuals.avatars,
            )
            .render(self.commands, root),
        }
    }

    fn sakura_open(&self) -> bool {
        #[cfg(not(target_os = "android"))]
        {
            self.visuals.sakura.open
        }
        #[cfg(target_os = "android")]
        {
            false
        }
    }

    fn render_overlays(&mut self, root: Entity) {
        #[cfg(not(target_os = "android"))]
        render_startup_dialog(self.commands, root, &self.visuals.sakura, &self.visuals.ui);
        if self.ui.lan.open {
            render_lan_rooms(
                self.commands,
                root,
                &self.visuals.lan,
                &self.ui.lan,
                &self.visuals.ui,
            );
        }
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

#[cfg(test)]
mod tests {
    use super::ScreenComposition;
    use bevy::ecs::system::SystemState;
    use bevy::prelude::*;

    #[test]
    fn screen_resources_have_no_conflicting_borrows() {
        let mut world = World::new();
        // Initialization checks nested SystemParams without loading assets or a window.
        let _state = SystemState::<ScreenComposition>::new(&mut world);
    }
}
