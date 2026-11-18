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
    pub accept: Option<UiAction>,
    pub cancel: Option<UiAction>,
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
        self.open_request(ConfirmationRequest {
            title,
            message,
            accept: Some(accept),
            cancel: None,
            accept_label,
            accept_variant,
        });
    }

    pub fn dismiss_if(&mut self, predicate: impl FnOnce(&UiAction) -> bool) -> bool {
        if self.open
            && self
                .request
                .as_ref()
                .and_then(|request| request.accept.as_ref())
                .is_some_and(predicate)
        {
            self.open = false;
            true
        } else {
            false
        }
    }

    pub fn is_busy(&self) -> bool {
        self.request.is_some()
    }

    pub fn show_choice(
        &mut self,
        title: String,
        message: String,
        accept: UiAction,
        cancel: UiAction,
        accept_label: &'static str,
    ) {
        self.open_request(ConfirmationRequest {
            title,
            message,
            accept: Some(accept),
            cancel: Some(cancel),
            accept_label,
            accept_variant: CozyButtonVariant::Cool,
        });
    }

    pub fn inform(&mut self, title: String, message: String) {
        self.open_request(ConfirmationRequest {
            title,
            message,
            accept: None,
            cancel: None,
            accept_label: "知道了",
            accept_variant: CozyButtonVariant::Neutral,
        });
    }

    fn open_request(&mut self, request: ConfirmationRequest) {
        if self.request.is_none() {
            self.request = Some(request);
            self.open = true;
            self.progress = 0.0;
        }
    }
}
