use super::actions::dispatch_achievements_actions;
use super::input::update_achievement_category_hover;
use super::state::AchievementsScroll;
use crate::app::runtime::ClientUpdateSet;
use crate::app::shell::{UiActionSet, scroll_modal_content};
use bevy::prelude::*;

pub(crate) struct AchievementsPagePlugin;

impl Plugin for AchievementsPagePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            scroll_modal_content::<AchievementsScroll>.in_set(ClientUpdateSet::Input),
        )
        .add_systems(Update, dispatch_achievements_actions.in_set(UiActionSet))
        .add_systems(
            Update,
            update_achievement_category_hover.in_set(ClientUpdateSet::Animate),
        );
    }
}
