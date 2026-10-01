//! Table appearance settings.

use crate::app::shell::SettingsUiAction;

use super::super::super::{UiAction, add_cozy_button};
use super::slider::SettingsSlider;
use crate::app::presentation::{MUTED, TEXT, TableAppearanceSetting, add_text, spawn_node};
use crate::app::runtime::{AppearancePreferences, UiAssets};
use bevy::prelude::*;
use bevy::ui::VisualBox;

pub(super) struct TableAppearanceSettings<'a> {
    form: &'a AppearancePreferences,
    assets: &'a UiAssets,
}

impl<'a> TableAppearanceSettings<'a> {
    pub(super) fn new(form: &'a AppearancePreferences, assets: &'a UiAssets) -> Self {
        Self { form, assets }
    }

    pub(super) fn render(&self, commands: &mut Commands, parent: Entity) {
        let form = self.form;
        let assets = self.assets;
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
            UiAction::Settings(SettingsUiAction::ChooseTableFelt),
            assets,
            px(110),
            38.0,
        );
        if form.table_felt_path.is_some() {
            add_cozy_button(
                commands,
                path_box,
                "恢复默认",
                UiAction::Settings(SettingsUiAction::UseDefaultTableFelt),
                assets,
                px(110),
                38.0,
            );
        }
        for setting in [
            TableAppearanceSetting::Brightness,
            TableAppearanceSetting::Vignette,
        ] {
            SettingsSlider::new(form, assets).render(commands, parent, setting);
        }
    }
}
