use super::dispatch_navigation_actions;
use crate::app::shell::UiActionSet;
use bevy::prelude::*;

pub(crate) struct NavigationPlugin;

impl Plugin for NavigationPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, dispatch_navigation_actions.in_set(UiActionSet));
    }
}
