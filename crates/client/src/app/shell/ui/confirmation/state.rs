use crate::app::shell::UiAction;
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
}

#[derive(Default, Resource)]
pub(crate) struct ConfirmationDialog {
    pub(super) request: Option<ConfirmationRequest>,
    pub(super) open: bool,
    pub(super) progress: f32,
}

impl ConfirmationDialog {
    #[cfg(target_os = "android")]
    pub fn is_open(&self) -> bool {
        self.open
    }

    pub fn show(&mut self, title: String, message: String, accept: UiAction) {
        if self.request.is_none() {
            self.request = Some(ConfirmationRequest {
                title,
                message,
                accept,
            });
            self.open = true;
            self.progress = 0.0;
        }
    }
}
