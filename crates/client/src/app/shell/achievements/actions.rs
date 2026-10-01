use super::AchievementCategory;
use crate::app::shell::{
    DomainUiAction, PressedUiAction, UiAction, UiActionHandler, UiState, dispatch_domain_actions,
};
use bevy::ecs::system::SystemParam;
use bevy::prelude::*;

#[derive(Clone)]
pub(crate) enum AchievementUiAction {
    SelectCategory(AchievementCategory),
}

impl DomainUiAction for AchievementUiAction {
    fn extract(action: &UiAction) -> Option<&Self> {
        let UiAction::Achievements(action) = action else {
            return None;
        };
        Some(action)
    }
}

#[derive(SystemParam)]
pub(super) struct AchievementUiActionContext<'w> {
    ui: ResMut<'w, UiState>,
}

pub(super) fn dispatch_achievements_actions(
    mut actions: MessageReader<PressedUiAction>,
    mut context: AchievementUiActionContext,
) {
    dispatch_domain_actions::<AchievementUiAction, _>(&mut actions, &mut context);
}

impl UiActionHandler<AchievementUiActionContext<'_>> for AchievementUiAction {
    fn handle(&self, context: &mut AchievementUiActionContext<'_>) {
        let ui = &mut *context.ui;
        match self {
            AchievementUiAction::SelectCategory(category) => {
                ui.achievements.category = *category;
            }
        }
    }
}
