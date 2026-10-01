use super::ProfileGameTab;
use crate::app::shell::{
    DomainUiAction, PressedUiAction, UiAction, UiActionHandler, UiState, dispatch_domain_actions,
};
use bevy::ecs::system::SystemParam;
use bevy::prelude::*;

#[derive(Clone)]
pub(crate) enum ProfileUiAction {
    SelectGameTab(ProfileGameTab),
}

impl DomainUiAction for ProfileUiAction {
    fn extract(action: &UiAction) -> Option<&Self> {
        let UiAction::Profile(action) = action else {
            return None;
        };
        Some(action)
    }
}

#[derive(SystemParam)]
pub(super) struct ProfileUiActionContext<'w> {
    ui: ResMut<'w, UiState>,
}

pub(super) fn dispatch_profile_actions(
    mut actions: MessageReader<PressedUiAction>,
    mut context: ProfileUiActionContext,
) {
    dispatch_domain_actions::<ProfileUiAction, _>(&mut actions, &mut context);
}

impl UiActionHandler<ProfileUiActionContext<'_>> for ProfileUiAction {
    fn handle(&self, context: &mut ProfileUiActionContext<'_>) {
        let ui = &mut *context.ui;
        match self {
            ProfileUiAction::SelectGameTab(tab) => ui.profile.game_tab = *tab,
        }
    }
}
