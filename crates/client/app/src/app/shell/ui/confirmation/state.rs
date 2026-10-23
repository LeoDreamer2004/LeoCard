use crate::app::shell::{CozyButtonVariant, UiAction};
use bevy::prelude::Resource;

#[derive(Clone)]
pub(crate) enum ConfirmationUiAction {
    Accept,
    Cancel,
}

pub(super) struct ConfirmationRequest {
    pub title: String,
    pub message: String,
    pub accept: UiAction,
    pub accept_label: &'static str,
    pub accept_variant: CozyButtonVariant,
}

#[derive(Default, Resource)]
pub(crate) struct ConfirmationDialog {
    pub(super) request: Option<ConfirmationRequest>,
    pub(super) open: bool,
    pub(super) progress: f32,
}

impl ConfirmationDialog {
    pub fn is_open(&self) -> bool {
        self.open
    }

    pub fn show(&mut self, title: String, message: String, accept: UiAction) {
        self.show_with_button(title, message, accept, "确认", CozyButtonVariant::Cool);
    }

    pub fn show_with_button(
        &mut self,
        title: String,
        message: String,
        accept: UiAction,
        accept_label: &'static str,
        accept_variant: CozyButtonVariant,
    ) {
        if self.request.is_none() {
            self.request = Some(ConfirmationRequest {
                title,
                message,
                accept,
                accept_label,
                accept_variant,
            });
            self.open = true;
            self.progress = 0.0;
        }
    }
}
