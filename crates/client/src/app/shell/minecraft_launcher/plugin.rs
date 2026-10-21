use super::actions::{dispatch_launcher_actions, sync_launcher_inputs};
use super::input::{remember_launcher_scroll, scroll_launcher_lists};
use super::state::{LauncherState, TaskMessage, TaskOutcome};
use crate::app::presentation::TextInputSet;
use crate::app::runtime::ClientUpdateSet;
use crate::app::shell::{UiActionSet, UiState};
use bevy::prelude::*;

pub(crate) struct MinecraftLauncherPlugin;

impl Plugin for MinecraftLauncherPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<LauncherState>()
            .add_systems(Update, dispatch_launcher_actions.in_set(UiActionSet))
            .add_systems(Update, poll_launcher_tasks.in_set(ClientUpdateSet::Sync))
            .add_systems(Update, scroll_launcher_lists.in_set(ClientUpdateSet::Input))
            .add_systems(
                Update,
                remember_launcher_scroll.in_set(ClientUpdateSet::Sync),
            )
            .add_systems(
                PostUpdate,
                sync_launcher_inputs.in_set(TextInputSet::Publish),
            );
    }
}

fn poll_launcher_tasks(mut state: ResMut<LauncherState>, mut ui: ResMut<UiState>) {
    let messages = {
        let Ok(mut queue) = state.messages.lock() else {
            return;
        };
        std::mem::take(&mut *queue)
    };
    if messages.is_empty() {
        return;
    }
    for message in messages {
        match message {
            TaskMessage::Progress(status) => {
                if status.starts_with("前往 ") {
                    state.login_instruction = Some(status.clone());
                }
                state.status = status;
            }
            TaskMessage::Finished(result) => {
                state.busy = false;
                state.login_instruction = None;
                match result {
                    Ok(TaskOutcome::Versions(versions)) => {
                        state.versions = versions;
                        if state.selected_version.is_empty()
                            && let Some(version) = state.versions.first()
                        {
                            state.selected_version = version.id.clone();
                        }
                        state.status = format!("获取到 {} 个游戏版本", state.versions.len());
                    }
                    Ok(TaskOutcome::Installed(id)) => {
                        state.selected_version = id.clone();
                        state.refresh_installed();
                        state.status = format!("{id} 安装完成");
                    }
                    Ok(TaskOutcome::Resources(projects)) => {
                        state.projects = projects;
                        state.status = format!("找到 {} 个资源", state.projects.len());
                    }
                    Ok(TaskOutcome::ResourceInstalled(path)) => {
                        state.status = format!("资源已下载到 {}", path.display());
                    }
                    Ok(TaskOutcome::Launched(pid)) => {
                        state.status = format!("Minecraft 已启动，进程 {pid}");
                    }
                    Ok(TaskOutcome::LoggedIn(account)) => {
                        state.status = format!("已登录正版账号 {}", account.name);
                        state.account = Some(account);
                    }
                    Err(error) => state.status = error,
                }
            }
        }
    }
    ui.dirty = true;
}
