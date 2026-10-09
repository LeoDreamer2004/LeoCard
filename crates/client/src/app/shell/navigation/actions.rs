//! 协调个人资料、设置和成就页面的导航。

use super::super::{
    AchievementCategory, DomainUiAction, PageMotion, PlayerProfilePage, PressedUiAction,
    ProfileMotion, SettingsMotion, UiAction, UiActionHandler, UiState, dispatch_domain_actions,
};
use crate::app::runtime::ClientResource;
use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use leocard_client::ClientPhaseRef;

#[derive(Clone)]
pub(crate) enum NavigationUiAction {
    ToggleProfile,
    OpenPlayerProfile(Box<PlayerProfilePage>),
    ToggleSettings,
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
    client: Option<Res<'w, ClientResource>>,
    settings_motion: ResMut<'w, SettingsMotion>,
    profile_motion: ResMut<'w, ProfileMotion>,
    page_motion: ResMut<'w, PageMotion>,
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
        match self {
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
                context.page_motion.begin();
                ui.shop.open = !ui.shop.open;
                if ui.shop.open {
                    if let Some(client) = context.client.as_deref()
                        && matches!(client.0.model().phase(), ClientPhaseRef::Playing(_))
                        && let Some(rules) = client.0.model().game_rules()
                    {
                        ui.shop.category = rules.kind();
                    }
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
