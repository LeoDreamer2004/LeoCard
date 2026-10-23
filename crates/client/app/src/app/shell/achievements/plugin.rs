use super::actions::dispatch_achievements_actions;
use super::input::{scroll_achievements, update_achievement_category_hover};
use crate::app::runtime::ClientUpdateSet;
use crate::app::shell::UiActionSet;
use bevy::prelude::*;

pub(crate) struct AchievementsPagePlugin;

impl Plugin for AchievementsPagePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, scroll_achievements.in_set(ClientUpdateSet::Input))
            .add_systems(Update, dispatch_achievements_actions.in_set(UiActionSet))
            .add_systems(
                Update,
                update_achievement_category_hover.in_set(ClientUpdateSet::Animate),
            );
    }
}
