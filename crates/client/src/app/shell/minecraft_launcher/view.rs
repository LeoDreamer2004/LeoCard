use super::actions::{LauncherInput, LauncherUiAction};
use super::input::LauncherScroll;
use super::state::{LauncherState, LauncherTab};
use crate::app::presentation::{
    MUTED, Scrollbar, ScrollbarSkin, TEXT, TextInput, UiComposer, add_text, spawn_node,
};
use crate::app::runtime::UiAssets;
use crate::app::shell::{
    CozyButtonVariant, UiAction, add_cozy_button_variant, add_cozy_disabled_button,
    add_page_back_title,
};
use bevy::prelude::*;
use bevy::ui::RelativeCursorPosition;
use leocard_minecraft_launcher::{ResourceKind, ResourceSource};

pub(crate) struct LauncherPage<'a> {
    pub state: &'a LauncherState,
    pub assets: &'a UiAssets,
}

impl LauncherPage<'_> {
    pub fn render(&self, commands: &mut UiComposer, root: Entity) {
        let canvas = spawn_node(
            commands,
            root,
            Node {
                width: percent(100),
                flex_grow: 1.0,
                min_height: px(0),
                justify_content: JustifyContent::Center,
                ..default()
            },
            None,
        );
        let page = spawn_node(
            commands,
            canvas,
            Node {
                width: percent(100),
                max_width: px(1180),
                min_height: px(0),
                padding: UiRect::all(px(22)),
                flex_direction: FlexDirection::Column,
                row_gap: px(14),
                ..default()
            },
            None,
        );
        let heading = spawn_node(
            commands,
            page,
            Node {
                width: percent(100),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::SpaceBetween,
                ..default()
            },
            None,
        );
        add_page_back_title(
            commands,
            heading,
            "Minecraft 启动器",
            UiAction::Launcher(LauncherUiAction::Close),
            self.assets,
        );
        add_text(
            commands,
            page,
            if self.state.login_instruction.is_some() {
                "等待微软账号授权"
            } else {
                &self.state.status
            },
            14.0,
            MUTED,
            self.assets,
        );
        if let Some(instruction) = &self.state.login_instruction {
            add_text(commands, page, instruction, 16.0, TEXT, self.assets);
        }
        let tabs = row(commands, page, 12.0);
        self.button(
            commands,
            tabs,
            "游戏版本",
            LauncherUiAction::SelectTab(LauncherTab::Versions),
            self.state.tab == LauncherTab::Versions,
            false,
            px(140),
        );
        self.button(
            commands,
            tabs,
            "资源下载",
            LauncherUiAction::SelectTab(LauncherTab::Resources),
            self.state.tab == LauncherTab::Resources,
            false,
            px(140),
        );
        let body = spawn_node(
            commands,
            page,
            Node {
                width: percent(100),
                min_height: px(0),
                flex_grow: 1.0,
                column_gap: px(18),
                ..default()
            },
            None,
        );
        match self.state.tab {
            LauncherTab::Versions => self.render_versions(commands, body),
            LauncherTab::Resources => self.render_resources(commands, body),
        }
    }

    fn render_versions(&self, commands: &mut UiComposer, body: Entity) {
        let list = list_panel(commands, body, 0.6);
        let search = row(commands, list, 8.0);
        input(
            commands,
            search,
            "mc.version.query",
            &self.state.version_query,
            "搜索版本",
            LauncherInput::VersionQuery,
            self.assets,
        );
        self.button(
            commands,
            search,
            "刷新",
            LauncherUiAction::RefreshVersions,
            false,
            self.state.busy,
            px(78),
        );
        let entries = scroll_list(
            commands,
            list,
            "minecraft-versions",
            LauncherScroll::Versions,
            self.state.version_scroll_y,
        );
        for version in &self.state.installed {
            let label = format!(
                "{}  已安装{}",
                version.id,
                version
                    .loader
                    .map_or("".to_owned(), |loader| format!(" · {}", loader.label()))
            );
            self.button(
                commands,
                entries,
                &label,
                LauncherUiAction::SelectVersion(version.id.clone()),
                self.state.selected_version == version.id,
                false,
                percent(100),
            );
        }
        for version in self
            .state
            .versions
            .iter()
            .filter(|version| version.id.contains(self.state.version_query.trim()))
            .filter(|version| !version.installed)
        {
            let label = format!(
                "{}  {}{}",
                version.id,
                if version.kind == "release" {
                    "正式版"
                } else {
                    "快照"
                },
                if version.installed { "  已安装" } else { "" }
            );
            self.button(
                commands,
                entries,
                &label,
                LauncherUiAction::SelectVersion(version.id.clone()),
                self.state.selected_version == version.id,
                false,
                percent(100),
            );
        }
        let detail_panel = list_panel(commands, body, 0.4);
        let detail = scroll_list(
            commands,
            detail_panel,
            "minecraft-version-detail",
            LauncherScroll::VersionDetail,
            self.state.detail_scroll_y,
        );
        add_text(commands, detail, "版本管理", 24.0, TEXT, self.assets);
        if !self.state.selected_version.is_empty() {
            add_text(
                commands,
                detail,
                &self.state.selected_version,
                17.0,
                TEXT,
                self.assets,
            );
        }
        add_text(commands, detail, "离线玩家名", 14.0, MUTED, self.assets);
        input(
            commands,
            detail,
            "mc.player.name",
            &self.state.player_name,
            "Player",
            LauncherInput::PlayerName,
            self.assets,
        );
        if let Some(account) = &self.state.account {
            add_text(
                commands,
                detail,
                format!("正版账号：{}", account.name),
                14.0,
                TEXT,
                self.assets,
            );
            self.button(
                commands,
                detail,
                "退出微软账号",
                LauncherUiAction::LogoutMicrosoft,
                false,
                self.state.busy,
                percent(100),
            );
        } else {
            self.button(
                commands,
                detail,
                "微软账号正版登录",
                LauncherUiAction::LoginMicrosoft,
                false,
                self.state.busy,
                percent(100),
            );
            add_text(
                commands,
                detail,
                "登录时按顶部提示打开微软授权页并输入代码。",
                12.0,
                MUTED,
                self.assets,
            );
        }
        let selected = self.state.selected_version.clone();
        let vanilla = self
            .state
            .versions
            .iter()
            .any(|version| version.id == selected);
        if vanilla {
            self.button(
                commands,
                detail,
                "安装原版",
                LauncherUiAction::InstallVanilla(selected.clone()),
                false,
                self.state.busy,
                percent(100),
            );
            self.button(
                commands,
                detail,
                "安装 Fabric",
                LauncherUiAction::InstallFabric(selected.clone()),
                false,
                self.state.busy,
                percent(100),
            );
            self.button(
                commands,
                detail,
                "安装 Forge",
                LauncherUiAction::InstallForge(selected.clone()),
                false,
                self.state.busy,
                percent(100),
            );
            self.button(
                commands,
                detail,
                "安装 NeoForge",
                LauncherUiAction::InstallNeoForge(selected.clone()),
                false,
                self.state.busy,
                percent(100),
            );
        }
        self.button(
            commands,
            detail,
            "启动游戏",
            LauncherUiAction::Launch,
            true,
            self.state.busy || !self.state.selected_installed(),
            percent(100),
        );
        add_text(
            commands,
            detail,
            "需要本机 Java。可用 LEOCARD_JAVA_PATH 指定运行时。",
            13.0,
            MUTED,
            self.assets,
        );
        if let Some(launcher) = &self.state.launcher {
            add_text(
                commands,
                detail,
                format!("文件目录：{}", launcher.root().display()),
                12.0,
                MUTED,
                self.assets,
            );
        }
    }

    fn render_resources(&self, commands: &mut UiComposer, body: Entity) {
        let browser = list_panel(commands, body, 0.65);
        let sources = row(commands, browser, 8.0);
        for source in [ResourceSource::Modrinth, ResourceSource::CurseForge] {
            self.button(
                commands,
                sources,
                source.label(),
                LauncherUiAction::SelectSource(source),
                self.state.resource_source == source,
                self.state.busy,
                px(130),
            );
        }
        let kinds = row(commands, browser, 8.0);
        for kind in [
            ResourceKind::Mod,
            ResourceKind::ResourcePack,
            ResourceKind::Shader,
            ResourceKind::DataPack,
        ] {
            self.button(
                commands,
                kinds,
                kind.label(),
                LauncherUiAction::SelectKind(kind),
                self.state.resource_kind == kind,
                self.state.busy,
                px(100),
            );
        }
        let search = row(commands, browser, 8.0);
        input(
            commands,
            search,
            "mc.resource.query",
            &self.state.resource_query,
            "搜索资源",
            LauncherInput::ResourceQuery,
            self.assets,
        );
        self.button(
            commands,
            search,
            "搜索",
            LauncherUiAction::SearchResources,
            false,
            self.state.busy
                || !self.state.selected_installed()
                || (self.state.resource_kind == ResourceKind::Mod && !self.state.selected_loader()),
            px(80),
        );
        let entries = scroll_list(
            commands,
            browser,
            "minecraft-resources",
            LauncherScroll::Resources,
            self.state.resource_scroll_y,
        );
        for project in &self.state.projects {
            let card = spawn_node(
                commands,
                entries,
                Node {
                    width: percent(100),
                    padding: UiRect::all(px(10)),
                    flex_direction: FlexDirection::Column,
                    row_gap: px(4),
                    ..default()
                },
                Some(Color::srgba(0.16, 0.17, 0.22, 0.8)),
            );
            add_text(
                commands,
                card,
                format!("{}  ·  {} 下载", project.title, project.downloads),
                17.0,
                TEXT,
                self.assets,
            );
            add_text(
                commands,
                card,
                &project.description,
                12.0,
                MUTED,
                self.assets,
            );
            self.button(
                commands,
                card,
                "下载到当前版本",
                LauncherUiAction::InstallResource(project.project_id.clone()),
                false,
                self.state.busy,
                px(180),
            );
        }
        let side = list_panel(commands, body, 0.35);
        add_text(commands, side, "安装目标", 24.0, TEXT, self.assets);
        add_text(
            commands,
            side,
            if self.state.selected_version.is_empty() {
                "先到游戏版本页选择版本"
            } else {
                &self.state.selected_version
            },
            17.0,
            TEXT,
            self.assets,
        );
        if self.state.resource_kind == ResourceKind::Mod && !self.state.selected_loader() {
            add_text(
                commands,
                side,
                "Mod 需要 Fabric、Forge 或 NeoForge 版本，请先安装并选择。",
                14.0,
                MUTED,
                self.assets,
            );
        }
        add_text(
            commands,
            side,
            "资源可从 Modrinth 或 CurseForge 下载。CurseForge 需要 LEOCARD_CURSEFORGE_API_KEY。数据包下载后可移入世界目录启用。",
            13.0,
            MUTED,
            self.assets,
        );
    }

    #[expect(
        clippy::too_many_arguments,
        reason = "button action, visual state and width stay explicit"
    )]
    fn button(
        &self,
        commands: &mut UiComposer,
        parent: Entity,
        label: &str,
        action: LauncherUiAction,
        selected: bool,
        disabled: bool,
        width: Val,
    ) {
        if disabled {
            add_cozy_disabled_button(commands, parent, label, self.assets, width, 42.0);
        } else {
            add_cozy_button_variant(
                commands,
                parent,
                label,
                UiAction::Launcher(action),
                self.assets,
                width,
                42.0,
                if selected {
                    CozyButtonVariant::Primary
                } else {
                    CozyButtonVariant::Neutral
                },
            );
        }
    }
}

fn row(commands: &mut UiComposer, parent: Entity, gap: f32) -> Entity {
    spawn_node(
        commands,
        parent,
        Node {
            width: percent(100),
            align_items: AlignItems::Center,
            column_gap: px(gap),
            ..default()
        },
        None,
    )
}

fn list_panel(commands: &mut UiComposer, parent: Entity, grow: f32) -> Entity {
    spawn_node(
        commands,
        parent,
        Node {
            min_width: px(0),
            min_height: px(0),
            flex_basis: px(0),
            flex_grow: grow,
            padding: UiRect::all(px(16)),
            flex_direction: FlexDirection::Column,
            row_gap: px(10),
            ..default()
        },
        Some(Color::srgba(0.10, 0.12, 0.16, 0.85)),
    )
}

fn scroll_list(
    commands: &mut UiComposer,
    parent: Entity,
    key: &str,
    kind: LauncherScroll,
    scroll_y: f32,
) -> Entity {
    let viewport = spawn_node(
        commands,
        parent,
        Node {
            width: percent(100),
            min_height: px(0),
            flex_grow: 1.0,
            flex_basis: px(0),
            column_gap: px(8),
            ..default()
        },
        None,
    );
    let list = commands.keyed_child(
        viewport,
        key,
        (
            Node {
                min_width: px(0),
                flex_basis: px(0),
                flex_grow: 1.0,
                min_height: px(0),
                flex_direction: FlexDirection::Column,
                row_gap: px(7),
                overflow: Overflow::scroll_y(),
                ..default()
            },
            kind,
            RelativeCursorPosition::default(),
            ScrollPosition(Vec2::new(0.0, scroll_y)),
        ),
    );
    Scrollbar::spawn(
        commands,
        viewport,
        list,
        ScrollbarSkin::Colored(Color::srgb(0.70, 0.75, 0.87)),
    );
    list
}

fn input(
    commands: &mut UiComposer,
    parent: Entity,
    key: &'static str,
    value: &str,
    placeholder: &'static str,
    kind: LauncherInput,
    assets: &UiAssets,
) {
    let mut field = TextInput::new(key, value);
    field.placeholder = placeholder;
    field.max_characters = 80;
    let editor = field.spawn(
        commands,
        parent,
        Node {
            min_width: px(0),
            flex_basis: px(0),
            flex_grow: 1.0,
            height: px(42),
            padding: UiRect::axes(px(12), px(8)),
            ..default()
        },
        assets,
    );
    commands.entity(editor).insert(kind);
}
