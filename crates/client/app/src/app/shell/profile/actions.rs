use super::scroll::ProfileArchiveDrag;
use super::{ProfileGameTab, ProfileGameTabButton};
use crate::app::shell::{
    DomainUiAction, PressedUiAction, UiAction, UiActionHandler, UiState, dispatch_domain_actions,
};
use bevy::ecs::system::SystemParam;
use bevy::picking::events::PointerDrag;
use bevy::prelude::*;
use bevy::ui_widgets::Activate;

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

pub(super) fn activate_profile_tab(
    event: On<Activate>,
    tabs: Query<&ProfileGameTabButton>,
    drag: Res<ProfileArchiveDrag>,
    mut actions: MessageWriter<PressedUiAction>,
    mut ui: ResMut<UiState>,
) {
    if !drag.moved
        && let Ok(tab) = tabs.get(event.entity)
    {
        actions.write(PressedUiAction(UiAction::Profile(
            ProfileUiAction::SelectGameTab(tab.0),
        )));
        ui.dirty = true;
    }
}

pub(super) fn observe_profile_tab_drag(
    event: On<PointerDrag>,
    tabs: Query<(), With<ProfileGameTabButton>>,
    mut drag: ResMut<ProfileArchiveDrag>,
) {
    if tabs.contains(event.entity) && event.distance.length() > 8.0 {
        drag.moved = true;
    }
}
