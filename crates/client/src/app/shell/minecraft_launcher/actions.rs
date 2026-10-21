use super::state::{LauncherState, LauncherTab, TaskMessage, TaskOutcome};
use crate::app::shell::{DomainUiAction, PressedUiAction, UiAction, UiState};
use bevy::prelude::*;
use bevy::text::EditableText;
use leocard_minecraft_launcher::{ResourceKind, ResourceSource};
use std::thread;
use std::time::Duration;

#[derive(Clone)]
pub(crate) enum LauncherUiAction {
    Show,
    Close,
    SelectTab(LauncherTab),
    SelectVersion(String),
    RefreshVersions,
    InstallVanilla(String),
    InstallFabric(String),
    InstallForge(String),
    InstallNeoForge(String),
    LoginMicrosoft,
    LogoutMicrosoft,
    Launch,
    SelectKind(ResourceKind),
    SelectSource(ResourceSource),
    SearchResources,
    InstallResource(String),
}

impl DomainUiAction for LauncherUiAction {
    fn extract(action: &UiAction) -> Option<&Self> {
        let UiAction::Launcher(action) = action else {
            return None;
        };
        Some(action)
    }
}

pub(crate) fn dispatch_launcher_actions(
    mut actions: MessageReader<PressedUiAction>,
    mut state: ResMut<LauncherState>,
    mut ui: ResMut<UiState>,
) {
    for pressed in actions.read() {
        let Some(action) = LauncherUiAction::extract(&pressed.0) else {
            continue;
        };
        match action {
            LauncherUiAction::Show => {
                state.open = true;
                state.refresh_installed();
                ui.shop.open = false;
                ui.achievements.open = false;
                ui.settings.open = false;
                ui.profile.open = false;
                if state.versions.is_empty() && !state.busy {
                    start_task(&mut state, "获取 Minecraft 版本", |launcher, _| {
                        launcher.versions().map(TaskOutcome::Versions)
                    });
                }
            }
            LauncherUiAction::Close => state.open = false,
            LauncherUiAction::SelectTab(tab) => state.tab = *tab,
            LauncherUiAction::SelectVersion(version) => {
                state.selected_version = version.clone();
                state.projects.clear();
                state.resource_scroll_y = 0.0;
            }
            LauncherUiAction::RefreshVersions => {
                start_task(&mut state, "获取 Minecraft 版本", |launcher, _| {
                    launcher.versions().map(TaskOutcome::Versions)
                });
            }
            LauncherUiAction::InstallVanilla(version) => {
                let version = version.clone();
                start_task(&mut state, "安装游戏", move |launcher, progress| {
                    launcher.install_vanilla(&version, progress)?;
                    Ok(TaskOutcome::Installed(version))
                });
            }
            LauncherUiAction::InstallFabric(version) => {
                let version = version.clone();
                start_task(&mut state, "安装 Fabric", move |launcher, progress| {
                    launcher
                        .install_fabric(&version, progress)
                        .map(TaskOutcome::Installed)
                });
            }
            LauncherUiAction::InstallForge(version) => {
                let version = version.clone();
                start_task(&mut state, "安装 Forge", move |launcher, progress| {
                    launcher
                        .install_forge(&version, progress)
                        .map(TaskOutcome::Installed)
                });
            }
            LauncherUiAction::InstallNeoForge(version) => {
                let version = version.clone();
                start_task(&mut state, "安装 NeoForge", move |launcher, progress| {
                    launcher
                        .install_neoforge(&version, progress)
                        .map(TaskOutcome::Installed)
                });
            }
            LauncherUiAction::LoginMicrosoft => {
                start_task(&mut state, "申请微软登录码", |launcher, progress| {
                    launcher
                        .login_microsoft(progress)
                        .map(TaskOutcome::LoggedIn)
                });
            }
            LauncherUiAction::LogoutMicrosoft => {
                state.account = None;
                state.status = "已退出微软账号".to_owned();
            }
            LauncherUiAction::Launch => {
                let version = state.selected_version.clone();
                let name = state.player_name.clone();
                let account = state.account.clone();
                start_task(&mut state, "启动游戏", move |launcher, _| {
                    let mut process = if let Some(account) = &account {
                        launcher.launch_authenticated(&version, account)?
                    } else {
                        launcher.launch_offline(&version, &name)?
                    };
                    thread::sleep(Duration::from_secs(2));
                    if let Some(status) = process.try_wait().map_err(|error| error.to_string())? {
                        return Err(format!(
                            "Minecraft 启动后立即退出（{status}），请查看 {}",
                            launcher
                                .root()
                                .join("instances")
                                .join(&version)
                                .join("launcher.log")
                                .display()
                        ));
                    }
                    Ok(TaskOutcome::Launched(process.id()))
                });
            }
            LauncherUiAction::SelectKind(kind) => {
                if !state.busy {
                    state.resource_kind = *kind;
                    state.projects.clear();
                    state.resource_scroll_y = 0.0;
                }
            }
            LauncherUiAction::SelectSource(source) => {
                if !state.busy {
                    state.resource_source = *source;
                    state.projects.clear();
                    state.resource_scroll_y = 0.0;
                }
            }
            LauncherUiAction::SearchResources => {
                let source = state.resource_source;
                let kind = state.resource_kind;
                let query = state.resource_query.clone();
                let version = state.selected_version.clone();
                let loader = state.loader();
                start_task(&mut state, "搜索资源", move |launcher, _| {
                    let game_version = launcher.game_version(&version)?;
                    launcher
                        .search_resources(source, kind, &query, &game_version, loader)
                        .map(TaskOutcome::Resources)
                });
            }
            LauncherUiAction::InstallResource(project_id) => {
                let source = state.resource_source;
                let kind = state.resource_kind;
                let project_id = project_id.clone();
                let version = state.selected_version.clone();
                start_task(&mut state, "下载资源", move |launcher, _| {
                    launcher
                        .install_resource(source, kind, &project_id, &version)
                        .map(TaskOutcome::ResourceInstalled)
                });
            }
        }
        ui.dirty = true;
    }
}

fn start_task(
    state: &mut LauncherState,
    title: &str,
    task: impl FnOnce(
        leocard_minecraft_launcher::MinecraftLauncher,
        &mut dyn FnMut(String),
    ) -> Result<TaskOutcome, String>
    + Send
    + 'static,
) {
    if state.busy {
        return;
    }
    let Some(launcher) = state.launcher.clone() else {
        return;
    };
    state.busy = true;
    state.status = title.to_owned();
    let messages = state.messages.clone();
    thread::spawn(move || {
        let mut progress = |status| {
            if let Ok(mut queue) = messages.lock() {
                queue.push(TaskMessage::Progress(status));
            }
        };
        let result = task(launcher, &mut progress);
        if let Ok(mut queue) = messages.lock() {
            queue.push(TaskMessage::Finished(result));
        }
    });
}

#[derive(Component, Clone, Copy)]
pub(super) enum LauncherInput {
    PlayerName,
    VersionQuery,
    ResourceQuery,
}

pub(crate) fn sync_launcher_inputs(
    inputs: Query<(&LauncherInput, &EditableText)>,
    mut state: ResMut<LauncherState>,
    mut ui: ResMut<UiState>,
) {
    for (kind, input) in &inputs {
        let value = match kind {
            LauncherInput::PlayerName => &mut state.player_name,
            LauncherInput::VersionQuery => &mut state.version_query,
            LauncherInput::ResourceQuery => &mut state.resource_query,
        };
        if input.value() != value.as_str() {
            *value = input.value().to_string();
            match kind {
                LauncherInput::VersionQuery => state.version_scroll_y = 0.0,
                LauncherInput::ResourceQuery => state.resource_scroll_y = 0.0,
                LauncherInput::PlayerName => {}
            }
            if !matches!(kind, LauncherInput::PlayerName) {
                ui.dirty = true;
            }
        }
    }
}
