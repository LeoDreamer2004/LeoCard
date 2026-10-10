use super::{UpdateManager, UpdateState, open_github_repository};
use crate::app::shell::{
    DomainUiAction, PressedUiAction, UiAction, UiActionHandler, dispatch_domain_actions,
};
use crate::updater::launch_installer;
use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use leocard_client::LocalPlayerProfile;
use leocard_protocol::ProfileId;

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
    #[cfg(not(target_os = "android"))]
    app_exit: MessageWriter<'w, AppExit>,
    profile: Res<'w, LocalPlayerProfile>,
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
            UpdateUiAction::RestartToUpdate => restart_to_update(
                updater,
                #[cfg(not(target_os = "android"))]
                &mut context.app_exit,
                context.profile.identity.profile_id(),
            ),
        }
    }
}

fn restart_to_update(
    updater: &mut UpdateManager,
    #[cfg(not(target_os = "android"))] app_exit: &mut MessageWriter<AppExit>,
    profile_id: ProfileId,
) {
    let UpdateState::Ready { staged, version } = &updater.state else {
        return;
    };
    match launch_installer(staged, &version.to_string(), profile_id) {
        Ok(()) => {
            updater.dialog_open = false;
            #[cfg(not(target_os = "android"))]
            app_exit.write(AppExit::Success);
        }
        Err(error) => {
            updater.state = UpdateState::Failed(error);
            updater.dialog_open = true;
        }
    }
}
