use super::{ConfirmationDialog, ConfirmationUiAction};
use crate::app::shell::{
    CozyModalKind, ModalAnimationSet, ModalAnimations, PressedUiAction, UiAction, UiActionSet,
    UiState, advance_modal,
};
use bevy::prelude::*;

pub(crate) struct ConfirmationPlugin;

impl Plugin for ConfirmationPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ConfirmationDialog>()
            .add_systems(Update, dispatch_confirmation.in_set(UiActionSet))
            .add_systems(
                Update,
                advance_confirmation.in_set(ModalAnimationSet::Progress),
            );
    }
}

fn dispatch_confirmation(
    mut messages: ParamSet<(
        MessageReader<PressedUiAction>,
        MessageWriter<PressedUiAction>,
    )>,
    mut dialog: ResMut<ConfirmationDialog>,
    mut ui: ResMut<UiState>,
) {
    let mut accepted = None;
    for message in messages.p0().read() {
        let UiAction::Confirmation(action) = &message.0 else {
            continue;
        };
        if !dialog.open {
            continue;
        }
        if matches!(action, ConfirmationUiAction::Accept) {
            accepted = dialog
                .request
                .as_ref()
                .map(|request| request.accept.clone());
        }
        dialog.open = false;
    }
    if let Some(action) = accepted {
        messages.p1().write(PressedUiAction(action));
        ui.dirty = true;
    }
}

fn advance_confirmation(
    time: Res<Time>,
    mut dialog: ResMut<ConfirmationDialog>,
    mut animations: ResMut<ModalAnimations>,
    mut ui: ResMut<UiState>,
) {
    if dialog.request.is_some() {
        let open = dialog.open;
        advance_modal(&mut dialog.progress, open, time.delta_secs());
        if !open && dialog.progress == 0.0 {
            dialog.request = None;
            ui.dirty = true;
        }
    }
    animations.set(CozyModalKind::Confirmation, dialog.progress);
}
