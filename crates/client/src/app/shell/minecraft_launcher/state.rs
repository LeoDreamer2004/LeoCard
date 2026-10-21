use bevy::prelude::*;
use leocard_minecraft_launcher::{
    InstalledVersion, MinecraftAccount, MinecraftLauncher, RemoteVersion, ResourceKind,
    ResourceProject, ResourceSource,
};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

#[derive(Clone, Copy, Eq, PartialEq)]
pub(crate) enum LauncherTab {
    Versions,
    Resources,
}

pub(crate) enum TaskOutcome {
    Versions(Vec<RemoteVersion>),
    Installed(String),
    Resources(Vec<ResourceProject>),
    ResourceInstalled(PathBuf),
    Launched(u32),
    LoggedIn(MinecraftAccount),
}

pub(crate) enum TaskMessage {
    Progress(String),
    Finished(Result<TaskOutcome, String>),
}

#[derive(Resource)]
pub(crate) struct LauncherState {
    pub open: bool,
    pub tab: LauncherTab,
    pub launcher: Option<MinecraftLauncher>,
    pub versions: Vec<RemoteVersion>,
    pub installed: Vec<InstalledVersion>,
    pub selected_version: String,
    pub version_query: String,
    pub version_scroll_y: f32,
    pub detail_scroll_y: f32,
    pub player_name: String,
    pub account: Option<MinecraftAccount>,
    pub resource_kind: ResourceKind,
    pub resource_source: ResourceSource,
    pub resource_query: String,
    pub projects: Vec<ResourceProject>,
    pub resource_scroll_y: f32,
    pub status: String,
    pub login_instruction: Option<String>,
    pub busy: bool,
    pub messages: Arc<Mutex<Vec<TaskMessage>>>,
}

impl Default for LauncherState {
    fn default() -> Self {
        let (launcher, status) = match MinecraftLauncher::new() {
            Ok(launcher) => (Some(launcher), "选择 Minecraft 版本开始".to_owned()),
            Err(error) => (None, error),
        };
        Self {
            open: false,
            tab: LauncherTab::Versions,
            launcher,
            versions: Vec::new(),
            installed: Vec::new(),
            selected_version: String::new(),
            version_query: String::new(),
            version_scroll_y: 0.0,
            detail_scroll_y: 0.0,
            player_name: "Player".to_owned(),
            account: None,
            resource_kind: ResourceKind::Mod,
            resource_source: ResourceSource::Modrinth,
            resource_query: String::new(),
            projects: Vec::new(),
            resource_scroll_y: 0.0,
            status,
            login_instruction: None,
            busy: false,
            messages: Arc::new(Mutex::new(Vec::new())),
        }
    }
}

impl LauncherState {
    pub fn selected_installed(&self) -> bool {
        self.installed
            .iter()
            .any(|version| version.id == self.selected_version)
    }

    pub fn selected_loader(&self) -> bool {
        self.installed
            .iter()
            .any(|version| version.id == self.selected_version && version.loader.is_some())
    }

    pub fn loader(&self) -> Option<leocard_minecraft_launcher::LoaderKind> {
        self.installed
            .iter()
            .find(|version| version.id == self.selected_version)
            .and_then(|version| version.loader)
    }

    pub fn refresh_installed(&mut self) {
        if let Some(launcher) = &self.launcher {
            match launcher.installed_versions() {
                Ok(installed) => self.installed = installed,
                Err(error) => self.status = error,
            }
            if self.selected_version.is_empty()
                && let Some(version) = self.installed.first()
            {
                self.selected_version = version.id.clone();
            }
        }
    }
}
