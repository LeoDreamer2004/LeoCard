//! 各游戏的规则区共用标题、模式选择与说明，配置项接在同一面板内。

use super::page::LobbyPageStyle;
use crate::app::presentation::{MUTED, TEXT, add_text};
use crate::app::runtime::UiAssets;
use crate::app::shell::CozyDropdown;
use bevy::prelude::*;
use leocard_ui::{UiComposer, spawn_node};

pub(crate) struct LobbyPageConfig {
    pub(super) style: LobbyPageStyle,
    title: String,
    description: Option<String>,
    mode: Option<(CozyDropdown, bool)>,
}

impl LobbyPageConfig {
    pub(crate) fn new(title: impl Into<String>) -> Self {
        Self {
            style: LobbyPageStyle::default(),
            title: title.into(),
            description: None,
            mode: None,
        }
    }

    pub(crate) fn style(mut self, style: LobbyPageStyle) -> Self {
        self.style = style;
        self
    }

    pub(crate) fn description(mut self, text: impl Into<String>) -> Self {
        self.description = Some(text.into());
        self
    }

    pub(crate) fn mode(mut self, dropdown: CozyDropdown, open: bool) -> Self {
        self.mode = Some((dropdown, open));
        self
    }

    pub(super) fn render(
        self,
        commands: &mut UiComposer,
        root: Entity,
        parent: Entity,
        can_configure: bool,
        assets: &UiAssets,
    ) {
        let row = spawn_node(
            commands,
            parent,
            Node {
                width: percent(100),
                flex_shrink: 0.0,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::SpaceBetween,
                column_gap: px(12),
                ..default()
            },
            None,
        );
        add_lobby_rules_heading(commands, row, self.title, assets);
        if let Some((dropdown, open)) = self.mode {
            dropdown.render(commands, root, row, can_configure, open, assets);
        }
        if let Some(description) = self.description {
            add_text(commands, parent, description, 13.0, MUTED, assets);
        }
    }
}

pub(super) fn add_lobby_rules_heading(
    commands: &mut UiComposer,
    parent: Entity,
    title: impl Into<String>,
    assets: &UiAssets,
) {
    let heading = spawn_node(
        commands,
        parent,
        Node {
            flex_direction: FlexDirection::Column,
            row_gap: px(3),
            ..default()
        },
        None,
    );
    add_text(commands, heading, title, 21.0, TEXT, assets);
    spawn_node(
        commands,
        heading,
        Node {
            width: px(76),
            height: px(2),
            ..default()
        },
        Some(Color::srgb(0.64, 0.59, 0.93)),
    );
}
