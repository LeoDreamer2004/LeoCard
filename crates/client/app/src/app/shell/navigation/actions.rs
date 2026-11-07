//! 协调个人资料、设置和成就页面的导航。

#[cfg(not(target_os = "android"))]
use super::super::SakuraState;
use super::super::{
    AchievementCategory, DomainUiAction, MusicMotion, PageMotion, PlayerProfilePage,
    PressedUiAction, ProfileMotion, SettingsMotion, UiAction, UiActionHandler, UiState,
    dispatch_domain_actions,
};
use crate::app::runtime::{ConnectionDraft, PageErrorState, save_connection_draft};
use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use leocard_client::PlayerEconomy;

#[derive(Clone)]
pub(crate) enum NavigationUiAction {
    ToggleProfile,
    OpenPlayerProfile(Box<PlayerProfilePage>),
    ToggleSettings,
    ToggleMusic,
    ToggleAchievements,
    ToggleShop,
}

impl DomainUiAction for NavigationUiAction {
    fn extract(action: &UiAction) -> Option<&Self> {
        let UiAction::Navigation(action) = action else {
            return None;
        };
        Some(action)
    }
}

#[derive(SystemParam)]
pub(crate) struct NavigationActionContext<'w> {
    ui: ResMut<'w, UiState>,
    settings_motion: ResMut<'w, SettingsMotion>,
    music_motion: ResMut<'w, MusicMotion>,
    profile_motion: ResMut<'w, ProfileMotion>,
    page_motion: ResMut<'w, PageMotion>,
    connection: Res<'w, ConnectionDraft>,
    economy: ResMut<'w, PlayerEconomy>,
    page_error: ResMut<'w, PageErrorState>,
    #[cfg(not(target_os = "android"))]
    sakura: ResMut<'w, SakuraState>,
}

pub(crate) fn dispatch_navigation_actions(
    mut actions: MessageReader<PressedUiAction>,
    mut context: NavigationActionContext,
) {
    dispatch_domain_actions::<NavigationUiAction, _>(&mut actions, &mut context);
}

impl UiActionHandler<NavigationActionContext<'_>> for NavigationUiAction {
    fn handle(&self, context: &mut NavigationActionContext<'_>) {
        let ui = &mut *context.ui;
        if ui.home.identity_open {
            context.page_error.error =
                save_connection_draft(&context.connection, &mut context.economy).err();
        }
        ui.home.identity_open = false;
        ui.home.join_open = false;
        if !matches!(self, NavigationUiAction::ToggleMusic) {
            context.music_motion.close();
        }
        match self {
            NavigationUiAction::ToggleMusic => {
                context.music_motion.toggle();
                if context.music_motion.target_open {
                    ui.settings.open = false;
                    context.settings_motion.target_open = false;
                    ui.profile.open = false;
                    context.profile_motion.target_open = false;
                    ui.home.identity_open = false;
                    ui.home.join_open = false;
                }
            }
            NavigationUiAction::ToggleProfile => {
                context.profile_motion.toggle(ui);
                if context.profile_motion.target_open {
                    ui.profile.player = None;
                    ui.settings.open = false;
                }
            }
            NavigationUiAction::OpenPlayerProfile(player_profile) => {
                context.profile_motion.open(ui);
                ui.profile.player = Some((**player_profile).clone());
                ui.social.interaction_menu_open = None;
                ui.settings.open = false;
            }
            NavigationUiAction::ToggleSettings => {
                context.settings_motion.toggle(ui);
                if context.settings_motion.target_open {
                    ui.profile.open = false;
                    ui.profile.player = None;
                    context.profile_motion.target_open = false;
                    context.profile_motion.progress = 0.0;
                }
            }
            NavigationUiAction::ToggleAchievements => {
                #[cfg(not(target_os = "android"))]
                {
                    context.sakura.open = false;
                }
                context.page_motion.begin();
                ui.achievements.open = !ui.achievements.open;
                if ui.achievements.open {
                    ui.shop.open = false;
                    ui.achievements.category = AchievementCategory::Mahjong;
                    ui.settings.open = false;
                    ui.profile.open = false;
                    context.settings_motion.target_open = false;
                    context.profile_motion.target_open = false;
                }
            }
            NavigationUiAction::ToggleShop => {
                #[cfg(not(target_os = "android"))]
                {
                    context.sakura.open = false;
                }
                context.page_motion.begin();
                ui.shop.open = !ui.shop.open;
                if ui.shop.open {
                    ui.achievements.open = false;
                    ui.settings.open = false;
                    ui.profile.open = false;
                    context.settings_motion.target_open = false;
                    context.profile_motion.target_open = false;
                }
            }
        }
    }
}
