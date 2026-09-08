//! 全局设置窗口与牌桌外观控制。

use super::*;

pub(super) struct SettingsModal<'a> {
    form: &'a ConnectionForm,
    updater: &'a UpdateManager,
    assets: &'a UiAssets,
}

impl<'a> SettingsModal<'a> {
    pub(super) fn new(
        form: &'a ConnectionForm,
        updater: &'a UpdateManager,
        assets: &'a UiAssets,
    ) -> Self {
        Self {
            form,
            updater,
            assets,
        }
    }

    pub(super) fn render(&self, commands: &mut Commands, root: Entity) {
        let form = self.form;
        let updater = self.updater;
        let assets = self.assets;
        let overlay = spawn_node(
            commands,
            root,
            Node {
                position_type: PositionType::Absolute,
                left: px(0),
                right: px(0),
                top: px(0),
                bottom: px(0),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
            Some(Color::srgba(0.005, 0.015, 0.012, 0.76)),
        );
        commands
            .entity(overlay)
            .insert((GlobalZIndex(2000), FocusPolicy::Block));
        let modal = add_panel(
            commands,
            overlay,
            Node {
                width: px(620),
                max_width: percent(92),
                flex_direction: FlexDirection::Column,
                row_gap: px(14),
                ..default()
            },
            PANEL,
            PanelSkin::Window,
            assets,
        );
        add_section_title(commands, modal, "游戏设置", assets);
        TableAppearanceSettings::new(form, assets).render(commands, modal);
        SoftwareUpdateSettings::new(updater, assets).render(commands, modal);
        SettingsActions::new(form, assets).render(commands, modal);
    }
}

struct TableAppearanceSettings<'a> {
    form: &'a ConnectionForm,
    assets: &'a UiAssets,
}

impl<'a> TableAppearanceSettings<'a> {
    fn new(form: &'a ConnectionForm, assets: &'a UiAssets) -> Self {
        Self { form, assets }
    }

    fn render(&self, commands: &mut Commands, parent: Entity) {
        let form = self.form;
        let assets = self.assets;
        add_text(commands, parent, "自定义桌布背景", 16.0, TEXT, assets);
        let path_box = spawn_node(
            commands,
            parent,
            Node {
                width: percent(100),
                min_height: px(54),
                padding: UiRect::all(px(10)),
                border: UiRect::all(px(1)),
                border_radius: BorderRadius::all(px(6)),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::SpaceBetween,
                column_gap: px(10),
                ..default()
            },
            Some(HEADER_BG),
        );
        commands.entity(path_box).insert(BorderColor::all(BORDER));
        let path_text = spawn_node(
            commands,
            path_box,
            Node {
                min_width: px(0),
                flex_grow: 1.0,
                ..default()
            },
            None,
        );
        add_text(
            commands,
            path_text,
            form.table_felt_path.as_ref().map_or_else(
                || "当前使用内置深绿色桌布".to_owned(),
                |path| format!("图片路径：{}", path.display()),
            ),
            13.0,
            if form.table_felt_path.is_some() {
                TEXT
            } else {
                MUTED
            },
            assets,
        );
        add_compact_button(
            commands,
            path_box,
            "选择图片",
            UiAction::Navigation(NavigationUiAction::ChooseTableFelt),
            assets,
        );
        for setting in [
            TableAppearanceSetting::Brightness,
            TableAppearanceSetting::Vignette,
            TableAppearanceSetting::Volume,
        ] {
            self.add_slider(commands, parent, setting);
        }
    }

    fn add_slider(&self, commands: &mut Commands, parent: Entity, setting: TableAppearanceSetting) {
        let form = self.form;
        let assets = self.assets;
        let fraction = table_appearance_fraction(setting, form);
        let group = spawn_node(
            commands,
            parent,
            Node {
                width: percent(100),
                flex_direction: FlexDirection::Column,
                row_gap: px(2),
                ..default()
            },
            None,
        );
        let label = add_text(
            commands,
            group,
            table_appearance_label(setting, form),
            14.0,
            TEXT,
            assets,
        );
        commands.entity(label).insert(TableAppearanceLabel(setting));
        let slider = commands
            .spawn((
                Button,
                TableAppearanceSlider(setting),
                RelativeCursorPosition::default(),
                Node {
                    width: percent(100),
                    height: px(34),
                    position_type: PositionType::Relative,
                    ..default()
                },
            ))
            .id();
        commands.entity(group).add_child(slider);
        let track = spawn_node(
            commands,
            slider,
            Node {
                position_type: PositionType::Absolute,
                left: px(0),
                right: px(0),
                top: px(13),
                height: px(8),
                border_radius: BorderRadius::all(px(4)),
                ..default()
            },
            Some(HEADER_BG),
        );
        let fill = spawn_node(
            commands,
            track,
            Node {
                position_type: PositionType::Absolute,
                left: px(0),
                top: px(0),
                width: percent(fraction * 100.0),
                height: percent(100),
                border_radius: BorderRadius::all(px(4)),
                ..default()
            },
            Some(Color::srgb(0.12, 0.48, 0.70)),
        );
        commands.entity(fill).insert(TableAppearanceIndicator {
            setting,
            part: TableAppearanceIndicatorPart::Fill,
        });
        let knob = spawn_node(
            commands,
            slider,
            Node {
                position_type: PositionType::Absolute,
                left: percent(fraction * 100.0),
                top: px(8),
                width: px(18),
                height: px(18),
                border: UiRect::all(px(2)),
                border_radius: BorderRadius::all(percent(50)),
                ..default()
            },
            Some(TEXT),
        );
        commands.entity(knob).insert((
            TableAppearanceIndicator {
                setting,
                part: TableAppearanceIndicatorPart::Knob,
            },
            BorderColor::all(Color::srgb(0.12, 0.48, 0.70)),
            UiTransform::from_translation(Val2::px(-9.0, 0.0)),
            FocusPolicy::Pass,
        ));
    }
}

struct SoftwareUpdateSettings<'a> {
    updater: &'a UpdateManager,
    assets: &'a UiAssets,
}

impl<'a> SoftwareUpdateSettings<'a> {
    fn new(updater: &'a UpdateManager, assets: &'a UiAssets) -> Self {
        Self { updater, assets }
    }

    fn render(self, commands: &mut Commands, parent: Entity) {
        let updater = self.updater;
        let assets = self.assets;
        let update_row = spawn_node(
            commands,
            parent,
            Node {
                width: percent(100),
                min_height: px(68),
                padding: UiRect::axes(px(12), px(10)),
                border: UiRect::all(px(1)),
                border_radius: BorderRadius::all(px(7)),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::SpaceBetween,
                flex_wrap: FlexWrap::Wrap,
                column_gap: px(12),
                row_gap: px(8),
                ..default()
            },
            Some(HEADER_BG.with_alpha(0.78)),
        );
        commands.entity(update_row).insert(BorderColor::all(BORDER));
        let version_text = spawn_node(
            commands,
            update_row,
            Node {
                min_width: px(180),
                flex_grow: 1.0,
                flex_direction: FlexDirection::Column,
                row_gap: px(3),
                ..default()
            },
            None,
        );
        add_text(commands, version_text, "软件更新", 16.0, TEXT, assets);
        add_text(
            commands,
            version_text,
            format!("当前版本 v{}", env!("CARGO_PKG_VERSION")),
            12.0,
            MUTED,
            assets,
        );
        let update_actions = spawn_node(
            commands,
            update_row,
            Node {
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Center,
                column_gap: px(10),
                ..default()
            },
            None,
        );
        add_github_repository_button(commands, update_actions, assets);
        add_green_update_button(
            commands,
            update_actions,
            settings_update_label(&updater.state),
            match updater.state {
                UpdateState::Ready { .. } => {
                    UiAction::Navigation(NavigationUiAction::RestartToUpdate)
                }
                _ => UiAction::Navigation(NavigationUiAction::StartUpdate),
            },
            assets,
        );
    }
}

struct SettingsActions<'a> {
    form: &'a ConnectionForm,
    assets: &'a UiAssets,
}

impl<'a> SettingsActions<'a> {
    fn new(form: &'a ConnectionForm, assets: &'a UiAssets) -> Self {
        Self { form, assets }
    }

    fn render(self, commands: &mut Commands, parent: Entity) {
        let actions = spawn_node(
            commands,
            parent,
            Node {
                width: percent(100),
                flex_direction: FlexDirection::Row,
                flex_wrap: FlexWrap::Wrap,
                justify_content: JustifyContent::FlexEnd,
                column_gap: px(10),
                row_gap: px(8),
                ..default()
            },
            None,
        );
        if self.form.table_felt_path.is_some() {
            add_action_button(
                commands,
                actions,
                "恢复默认",
                UiAction::Navigation(NavigationUiAction::UseDefaultTableFelt),
                ButtonKind::Secondary,
                self.assets,
            );
        }
        add_action_button(
            commands,
            actions,
            "关闭",
            UiAction::Navigation(NavigationUiAction::ToggleSettings),
            ButtonKind::Secondary,
            self.assets,
        );
    }
}
