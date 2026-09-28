//! 全局设置窗口与牌桌外观控制。
use super::super::{
    CozyButtonVariant, CozyModalBackdrop, CozyModalKind, CozyModalPanel, NavigationUiAction,
    SettingsTab, UiAction, UpdateManager, UpdateState, add_cozy_button, add_cozy_button_variant,
    add_cozy_button_with_icon, add_cozy_close_button, add_cozy_panel, settings_update_label,
    table_appearance_fraction, table_appearance_label,
};
use super::{
    CozySettingsSlider, SelectedSettingsTab, SettingsTabButton, cozy_backdrop_color,
    cozy_panel_transform,
};
use crate::app::presentation::{
    MUTED, TEXT, TableAppearanceIndicator, TableAppearanceLabel, TableAppearanceSetting,
    TableAppearanceSlider, add_text, spawn_node,
};
use crate::app::runtime::{AppearancePreferences, UiAssets};
use bevy::prelude::*;
use bevy::ui::{FocusPolicy, RelativeCursorPosition, VisualBox};

pub(crate) struct SettingsModal<'a> {
    form: &'a AppearancePreferences,
    updater: &'a UpdateManager,
    assets: &'a UiAssets,
}

impl<'a> SettingsModal<'a> {
    pub(crate) fn new(
        form: &'a AppearancePreferences,
        updater: &'a UpdateManager,
        assets: &'a UiAssets,
    ) -> Self {
        Self {
            form,
            updater,
            assets,
        }
    }

    pub(crate) fn render(
        &self,
        commands: &mut Commands,
        root: Entity,
        progress: f32,
        selected_tab: SettingsTab,
    ) {
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
            Some(cozy_backdrop_color(progress)),
        );
        commands.entity(overlay).insert((
            GlobalZIndex(2000),
            FocusPolicy::Block,
            CozyModalBackdrop(CozyModalKind::Settings),
        ));
        let modal = add_cozy_panel(
            commands,
            overlay,
            Node {
                width: px(680),
                max_width: percent(92),
                padding: UiRect::all(px(24)),
                flex_direction: FlexDirection::Column,
                row_gap: px(12),
                ..default()
            },
            assets,
        );
        commands.entity(modal).insert((
            CozyModalPanel(CozyModalKind::Settings),
            cozy_panel_transform(progress),
        ));
        let heading = spawn_node(
            commands,
            modal,
            Node {
                width: percent(100),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::SpaceBetween,
                ..default()
            },
            None,
        );
        add_text(commands, heading, "游戏设置", 28.0, TEXT, assets);
        add_cozy_close_button(
            commands,
            heading,
            UiAction::Navigation(NavigationUiAction::ToggleSettings),
            assets,
        );
        spawn_node(
            commands,
            modal,
            Node {
                width: px(96),
                height: px(2),
                ..default()
            },
            Some(Color::srgb(0.64, 0.59, 0.93)),
        );
        let tabbed = spawn_node(
            commands,
            modal,
            Node {
                width: percent(100),
                flex_direction: FlexDirection::Row,
                column_gap: px(14),
                ..default()
            },
            None,
        );
        let tabs = spawn_node(
            commands,
            tabbed,
            Node {
                width: px(100),
                min_width: px(100),
                padding: UiRect::top(px(18)),
                flex_direction: FlexDirection::Column,
                row_gap: px(6),
                ..default()
            },
            None,
        );
        for (tab, label) in [
            (SettingsTab::Appearance, "外观"),
            (SettingsTab::About, "关于"),
        ] {
            add_settings_tab(commands, tabs, tab, label, selected_tab, assets);
        }
        let mut content_image = ImageNode::new(assets.home.settings_page.clone()).with_mode(
            NodeImageMode::Sliced(TextureSlicer {
                border: BorderRect::all(20.0),
                center_scale_mode: SliceScaleMode::Stretch,
                sides_scale_mode: SliceScaleMode::Stretch,
                max_corner_scale: 1.0,
            }),
        );
        content_image.visual_box = VisualBox::BorderBox;
        content_image.color = Color::srgb(0.55, 0.54, 0.63);
        let content = commands
            .spawn((
                Node {
                    min_width: px(0),
                    min_height: px(390),
                    flex_grow: 1.0,
                    padding: UiRect::all(px(22)),
                    flex_direction: FlexDirection::Column,
                    row_gap: px(12),
                    ..default()
                },
                content_image,
            ))
            .id();
        commands.entity(tabbed).add_child(content);
        match selected_tab {
            SettingsTab::Appearance => {
                TableAppearanceSettings::new(form, assets).render(commands, content);
            }
            SettingsTab::About => {
                AboutSettings::new(updater, assets).render(commands, content);
            }
        }
    }
}

fn add_settings_tab(
    commands: &mut Commands,
    parent: Entity,
    tab: SettingsTab,
    label: &str,
    selected_tab: SettingsTab,
    assets: &UiAssets,
) {
    let selected = tab == selected_tab;
    let mut tab_image = ImageNode::new(if selected {
        assets.home.game_card_hover.clone()
    } else {
        assets.home.game_card.clone()
    })
    .with_mode(NodeImageMode::Sliced(TextureSlicer {
        border: BorderRect::all(16.0),
        center_scale_mode: SliceScaleMode::Stretch,
        sides_scale_mode: SliceScaleMode::Stretch,
        max_corner_scale: 1.0,
    }));
    tab_image.visual_box = VisualBox::BorderBox;
    let button = commands
        .spawn((
            Button,
            UiAction::Navigation(NavigationUiAction::SelectSettingsTab(tab)),
            Node {
                width: percent(100),
                height: px(48),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
            tab_image,
        ))
        .id();
    if selected {
        commands.entity(button).insert(SelectedSettingsTab);
    }
    commands.entity(parent).add_child(button);
    let text = add_text(
        commands,
        button,
        label,
        17.0,
        if selected {
            Color::srgb(0.85, 0.82, 1.0)
        } else {
            TEXT
        },
        assets,
    );
    commands
        .entity(button)
        .insert(SettingsTabButton { label: text });
}

struct TableAppearanceSettings<'a> {
    form: &'a AppearancePreferences,
    assets: &'a UiAssets,
}

impl<'a> TableAppearanceSettings<'a> {
    fn new(form: &'a AppearancePreferences, assets: &'a UiAssets) -> Self {
        Self { form, assets }
    }

    fn render(&self, commands: &mut Commands, parent: Entity) {
        let form = self.form;
        let assets = self.assets;
        self.add_slider(commands, parent, TableAppearanceSetting::Volume);
        spawn_node(
            commands,
            parent,
            Node {
                width: percent(100),
                height: px(1),
                margin: UiRect::vertical(px(3)),
                ..default()
            },
            Some(Color::srgba(0.70, 0.68, 0.78, 0.36)),
        );
        add_text(commands, parent, "桌面外观", 20.0, TEXT, assets);
        add_text(commands, parent, "桌布背景", 15.0, MUTED, assets);
        let mut path_image = ImageNode::new(assets.home.input.clone()).with_mode(
            NodeImageMode::Sliced(TextureSlicer {
                border: BorderRect::all(32.0),
                center_scale_mode: SliceScaleMode::Stretch,
                sides_scale_mode: SliceScaleMode::Stretch,
                max_corner_scale: 0.55,
            }),
        );
        path_image.visual_box = VisualBox::BorderBox;
        let path_box = commands
            .spawn((
                Node {
                    width: percent(100),
                    min_height: px(54),
                    padding: UiRect::axes(px(12), px(8)),
                    align_items: AlignItems::Center,
                    column_gap: px(10),
                    ..default()
                },
                path_image,
            ))
            .id();
        commands.entity(parent).add_child(path_box);
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
        add_cozy_button(
            commands,
            path_box,
            "选择图片",
            UiAction::Navigation(NavigationUiAction::ChooseTableFelt),
            assets,
            px(110),
            38.0,
        );
        if form.table_felt_path.is_some() {
            add_cozy_button(
                commands,
                path_box,
                "恢复默认",
                UiAction::Navigation(NavigationUiAction::UseDefaultTableFelt),
                assets,
                px(110),
                38.0,
            );
        }
        for setting in [
            TableAppearanceSetting::Brightness,
            TableAppearanceSetting::Vignette,
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
                row_gap: px(4),
                ..default()
            },
            None,
        );
        let label = add_text(
            commands,
            group,
            table_appearance_label(setting, form),
            15.0,
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
                    height: px(38),
                    position_type: PositionType::Relative,
                    ..default()
                },
            ))
            .id();
        commands.entity(group).add_child(slider);
        let mut track_image = ImageNode::new(assets.home.slider.clone()).with_mode(
            NodeImageMode::Sliced(TextureSlicer {
                border: BorderRect::all(32.0),
                center_scale_mode: SliceScaleMode::Stretch,
                sides_scale_mode: SliceScaleMode::Stretch,
                max_corner_scale: 0.375,
            }),
        );
        track_image.visual_box = VisualBox::BorderBox;
        let track = commands
            .spawn((
                Node {
                    position_type: PositionType::Absolute,
                    left: px(0),
                    right: px(0),
                    top: px(7),
                    height: px(24),
                    ..default()
                },
                track_image,
                FocusPolicy::Pass,
            ))
            .id();
        commands.entity(slider).add_child(track);
        let knob = commands
            .spawn((
                Node {
                    position_type: PositionType::Absolute,
                    left: percent(fraction * 100.0),
                    top: px(2),
                    width: px(14),
                    height: px(34),
                    ..default()
                },
                ImageNode::new(assets.home.slider_handle.clone()).with_mode(NodeImageMode::Stretch),
            ))
            .id();
        commands.entity(slider).add_child(knob);
        commands.entity(knob).insert((
            TableAppearanceIndicator(setting),
            CozySettingsSlider {
                owner: slider,
                track,
                hover: 0.0,
            },
            UiTransform::from_translation(Val2::px(-7.0, 0.0)),
            FocusPolicy::Pass,
        ));
    }
}

struct AboutSettings<'a> {
    updater: &'a UpdateManager,
    assets: &'a UiAssets,
}

impl<'a> AboutSettings<'a> {
    fn new(updater: &'a UpdateManager, assets: &'a UiAssets) -> Self {
        Self { updater, assets }
    }

    fn render(self, commands: &mut Commands, parent: Entity) {
        let identity = spawn_node(
            commands,
            parent,
            Node {
                width: percent(100),
                flex_grow: 1.0,
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                row_gap: px(8),
                ..default()
            },
            None,
        );
        add_text(
            commands,
            identity,
            "LeoCard",
            34.0,
            Color::srgb(0.85, 0.82, 1.0),
            self.assets,
        );
        add_text(
            commands,
            identity,
            "五种游戏，一张牌桌",
            16.0,
            TEXT,
            self.assets,
        );
        add_text(
            commands,
            identity,
            format!("版本 v{}", env!("CARGO_PKG_VERSION")),
            14.0,
            MUTED,
            self.assets,
        );
        SoftwareUpdateSettings::new(self.updater, self.assets).render(commands, parent);
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
        spawn_node(
            commands,
            parent,
            Node {
                width: percent(100),
                height: px(1),
                margin: UiRect::vertical(px(4)),
                ..default()
            },
            Some(Color::srgba(0.70, 0.68, 0.78, 0.36)),
        );
        let update_row = spawn_node(
            commands,
            parent,
            Node {
                width: percent(100),
                min_height: px(54),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::SpaceBetween,
                flex_wrap: FlexWrap::Wrap,
                column_gap: px(12),
                row_gap: px(8),
                ..default()
            },
            None,
        );
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
        add_text(commands, version_text, "软件更新", 20.0, TEXT, assets);
        add_text(
            commands,
            version_text,
            "查看项目或检查新版本",
            13.0,
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
        add_cozy_button_with_icon(
            commands,
            update_actions,
            "GitHub",
            UiAction::Navigation(NavigationUiAction::OpenGitHubRepository),
            assets,
            px(124),
            40.0,
            Some(assets.controls.github_mark.clone()),
        );
        add_cozy_button_variant(
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
            px(124),
            40.0,
            CozyButtonVariant::Cool,
        );
    }
}
