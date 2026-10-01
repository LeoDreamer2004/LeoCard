use super::{UpdateManager, UpdateState, open_github_repository};
use crate::app::shell::{
    DomainUiAction, PressedUiAction, UiAction, UiActionHandler, dispatch_domain_actions,
};
use crate::updater::launch_installer;
use bevy::ecs::system::SystemParam;
use bevy::prelude::*;

#[derive(Clone)]
pub(crate) enum UpdateUiAction {
    StartUpdate,
    OpenGitHubRepository,
    HideUpdateDialog,
    RestartToUpdate,
}

impl DomainUiAction for UpdateUiAction {
    fn rebuilds_ui(&self) -> bool {
        !matches!(self, Self::OpenGitHubRepository)
    }

    fn extract(action: &UiAction) -> Option<&Self> {
        let UiAction::Update(action) = action else {
            return None;
        };
        Some(action)
    }
}

#[derive(SystemParam)]
pub(super) struct UpdateUiActionContext<'w> {
    updater: ResMut<'w, UpdateManager>,
    app_exit: MessageWriter<'w, AppExit>,
}

pub(super) fn dispatch_update_actions(
    mut actions: MessageReader<PressedUiAction>,
    mut context: UpdateUiActionContext,
) {
    dispatch_domain_actions::<UpdateUiAction, _>(&mut actions, &mut context);
}

impl UiActionHandler<UpdateUiActionContext<'_>> for UpdateUiAction {
    fn handle(&self, context: &mut UpdateUiActionContext<'_>) {
        let updater = &mut *context.updater;
        match self {
            UpdateUiAction::StartUpdate => updater.begin_or_show(),
            UpdateUiAction::OpenGitHubRepository => {
                if let Err(error) = open_github_repository() {
                    warn!("{error}");
                }
            }
            UpdateUiAction::HideUpdateDialog => updater.dialog_open = false,
            UpdateUiAction::RestartToUpdate => restart_to_update(updater, &mut context.app_exit),
        }
    }
}

fn restart_to_update(updater: &mut UpdateManager, app_exit: &mut MessageWriter<AppExit>) {
    let UpdateState::Ready { staged, .. } = &updater.state else {
        return;
    };
    match launch_installer(staged) {
        Ok(()) => {
            app_exit.write(AppExit::Success);
        }
        Err(error) => {
            updater.state = UpdateState::Failed(error);
            updater.dialog_open = true;
        }
    }
}
