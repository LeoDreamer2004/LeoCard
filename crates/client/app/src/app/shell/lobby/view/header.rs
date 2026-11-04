//! 各游戏的规则区共用标题、模式选择与说明，配置项接在同一面板内。

use super::page::LobbyPageStyle;
use crate::app::presentation::{MUTED, add_text, add_title};
use crate::app::runtime::UiAssets;
use crate::app::shell::CozyDropdown;
use bevy::prelude::*;
use leocard_ui::{PageFrameEntities, UiComposer};

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
        frame: &PageFrameEntities,
        can_configure: bool,
        assets: &UiAssets,
    ) {
        add_lobby_rules_heading(commands, frame.heading, self.title, assets);
        if let Some((dropdown, open)) = self.mode {
            dropdown.render(commands, root, frame.heading, can_configure, open, assets);
        }
        if let Some(description) = self.description {
            add_text(commands, frame.content, description, 13.0, MUTED, assets);
        }
    }
}

pub(super) fn add_lobby_rules_heading(
    commands: &mut UiComposer,
    parent: Entity,
    title: impl Into<String>,
    assets: &UiAssets,
) {
    add_title(
        commands,
        parent,
        title,
        21.0,
        Color::srgb(0.13, 0.14, 0.17),
        assets,
    );
}
