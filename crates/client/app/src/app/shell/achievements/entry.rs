//! 成就图鉴的顶栏入口。
use crate::app::runtime::UiAssets;
use crate::app::shell::{NavigationUiAction, UiAction, add_page_icon_button};
use bevy::prelude::*;
use leocard_ui::UiComposer;

pub(crate) struct AchievementEntry<'a> {
    pub assets: &'a UiAssets,
}

impl AchievementEntry<'_> {
    pub(crate) fn render(&self, commands: &mut UiComposer, parent: Entity, scale: f32) {
        add_page_icon_button(
            commands,
            parent,
            UiAction::Navigation(NavigationUiAction::ToggleAchievements),
            self.assets,
            self.assets.achievements.icon.clone(),
            "成就",
            scale,
        );
    }
}
