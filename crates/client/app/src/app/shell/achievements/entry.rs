//! 成就图鉴的顶栏入口。
use crate::app::presentation::UiComposer;
use crate::app::runtime::UiAssets;
use crate::app::shell::{
    CozyButtonVariant, NavigationUiAction, UiAction, add_cozy_labeled_icon_button,
};
use bevy::prelude::*;

pub(crate) struct AchievementEntry<'a> {
    pub assets: &'a UiAssets,
}

impl AchievementEntry<'_> {
    pub(crate) fn render(&self, commands: &mut UiComposer, parent: Entity) {
        add_cozy_labeled_icon_button(
            commands,
            parent,
            UiAction::Navigation(NavigationUiAction::ToggleAchievements),
            self.assets,
            self.assets.achievements.icon.clone(),
            CozyButtonVariant::Neutral,
            "成就",
        );
    }
}
