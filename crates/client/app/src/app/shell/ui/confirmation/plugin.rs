use super::{ConfirmationDialog, ConfirmationUiAction};
use crate::app::runtime::{ClientUpdateSet, ServerNotification};
use crate::app::shell::{
    CozyModalKind, ModalAnimationSet, ModalAnimations, PressedUiAction, UiAction, UiActionSet,
    UiState, advance_modal,
};
use bevy::prelude::*;
use leocard_protocol::ServerEvent;
use std::collections::VecDeque;

pub(crate) struct ConfirmationPlugin;

impl Plugin for ConfirmationPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ConfirmationDialog>()
            .add_systems(Update, dispatch_confirmation.in_set(UiActionSet))
            .add_systems(Update, show_server_notices.in_set(ClientUpdateSet::Sync))
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
                .and_then(|request| request.accept.clone());
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

fn show_server_notices(
    mut messages: MessageReader<ServerNotification>,
    mut pending: Local<VecDeque<(String, String)>>,
    mut dialog: ResMut<ConfirmationDialog>,
    mut ui: ResMut<UiState>,
) {
    for notification in messages.read() {
        if let ServerEvent::Notice { title, message } = &notification.event {
            pending.push_back((title.clone(), message.clone()));
        }
    }
    if dialog.request.is_none()
        && let Some((title, message)) = pending.pop_front()
    {
        dialog.inform(title, message);
        ui.dirty = true;
    }
}
