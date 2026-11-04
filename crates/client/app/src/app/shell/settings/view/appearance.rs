//! Table and game appearance settings.

use super::section::add_settings_separator;
use super::slider::SettingsSlider;
use super::toggle::add_settings_toggle;
use crate::app::presentation::{MUTED, TEXT, TableAppearanceSetting, add_text, add_title};
use crate::app::runtime::{AppearancePreferences, TableSurfaceKind, UiAssets};
use crate::app::shell::{SettingsUiAction, UiAction, add_cozy_button};
use bevy::prelude::*;
use bevy::ui::VisualBox;
use leocard_ui::{UiComposer, spawn_node};

pub(super) struct TableAppearanceSettings<'a> {
    form: &'a AppearancePreferences,
    assets: &'a UiAssets,
}

impl<'a> TableAppearanceSettings<'a> {
    pub(super) fn new(form: &'a AppearancePreferences, assets: &'a UiAssets) -> Self {
        Self { form, assets }
    }

    pub(super) fn render(&self, commands: &mut UiComposer, parent: Entity) {
        let form = self.form;
        let assets = self.assets;
        #[cfg(not(target_os = "android"))]
        {
            add_title(commands, parent, "通知", 20.0, TEXT, assets);
            add_settings_toggle(
                commands,
                parent,
                "轮到我时弹窗提醒",
                form.turn_notifications,
                SettingsUiAction::ToggleTurnNotifications,
                assets,
            );
            add_settings_separator(commands, parent);
        }
        for (index, (kind, title)) in [
            (TableSurfaceKind::Cards, "扑克牌桌面"),
            (TableSurfaceKind::Mahjong, "麻将桌面"),
        ]
        .into_iter()
        .enumerate()
        {
            if index > 0 {
                add_settings_separator(commands, parent);
            }
            add_title(commands, parent, title, 20.0, TEXT, assets);
            self.render_table(commands, parent, kind);
        }
        add_settings_separator(commands, parent);
        add_title(commands, parent, "MC 竞猜", 20.0, TEXT, assets);
        add_settings_toggle(
            commands,
            parent,
            "随机背景",
            form.games.mc_guess_random_background,
            SettingsUiAction::ToggleMcGuessBackground,
            assets,
        );
    }

    fn render_table(&self, commands: &mut UiComposer, parent: Entity, kind: TableSurfaceKind) {
        let form = self.form;
        let assets = self.assets;
        let path = form.felt_path(kind);
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
        let path_box = commands.child(
            parent,
            (
                Node {
                    width: percent(100),
                    min_height: px(54),
                    padding: UiRect::axes(px(12), px(8)),
                    align_items: AlignItems::Center,
                    column_gap: px(10),
                    ..default()
                },
                path_image,
            ),
        );
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
            path.as_ref().map_or_else(
                || "当前使用默认牌桌背景".to_owned(),
                |path| format!("图片路径：{}", path.display()),
            ),
            13.0,
            if path.is_some() { TEXT } else { MUTED },
            assets,
        );
        add_cozy_button(
            commands,
            path_box,
            "选择图片",
            UiAction::Settings(SettingsUiAction::ChooseTableFelt(kind)),
            assets,
            px(110),
            38.0,
        );
        add_cozy_button(
            commands,
            path_box,
            "恢复默认",
            UiAction::Settings(SettingsUiAction::UseDefaultTableFelt(kind)),
            assets,
            px(110),
            38.0,
        );
        SettingsSlider::new(form, assets).render(
            commands,
            parent,
            TableAppearanceSetting::Brightness(kind),
        );
    }
}
