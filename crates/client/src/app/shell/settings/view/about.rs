//! Product information and update actions.

use crate::app::shell::UpdateUiAction;

use super::super::super::{
    CozyButtonVariant, UiAction, UpdateManager, UpdateState, add_cozy_button_variant,
    add_cozy_button_with_icon, settings_update_label,
};
use crate::app::presentation::{MUTED, TEXT, add_text, spawn_node};
use crate::app::runtime::UiAssets;
use bevy::prelude::*;

pub(super) struct AboutSettings<'a> {
    updater: &'a UpdateManager,
    assets: &'a UiAssets,
}

impl<'a> AboutSettings<'a> {
    pub(super) fn new(updater: &'a UpdateManager, assets: &'a UiAssets) -> Self {
        Self { updater, assets }
    }

    pub(super) fn render(self, commands: &mut Commands, parent: Entity) {
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
            UiAction::Update(UpdateUiAction::OpenGitHubRepository),
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
                UpdateState::Ready { .. } => UiAction::Update(UpdateUiAction::RestartToUpdate),
                _ => UiAction::Update(UpdateUiAction::StartUpdate),
            },
            assets,
            px(124),
            40.0,
            CozyButtonVariant::Cool,
        );
    }
}
