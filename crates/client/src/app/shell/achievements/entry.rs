//! 成就图鉴的顶栏入口。
use crate::app::presentation::{TEXT, add_text, spawn_node};
use crate::app::runtime::UiAssets;
use crate::app::shell::{CozyButtonVariant, NavigationUiAction, UiAction, add_cozy_icon_button};
use bevy::prelude::*;

pub(crate) struct AchievementEntry<'a> {
    pub assets: &'a UiAssets,
}

impl AchievementEntry<'_> {
    pub(crate) fn render(&self, commands: &mut Commands, parent: Entity) {
        let column = spawn_node(
            commands,
            parent,
            Node {
                width: px(44),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                row_gap: px(1),
                ..default()
            },
            None,
        );
        add_cozy_icon_button(
            commands,
            column,
            UiAction::Navigation(NavigationUiAction::ToggleAchievements),
            self.assets,
            self.assets.achievements.icon.clone(),
            CozyButtonVariant::Neutral,
        );
        add_text(commands, column, "成就", 12.0, TEXT, self.assets);
    }
}
