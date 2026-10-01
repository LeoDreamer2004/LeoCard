//! 成就图鉴的顶栏入口。
use crate::app::presentation::{ButtonTint, TEXT, add_text, spawn_node};
use crate::app::runtime::UiAssets;
use crate::app::shell::{NavigationUiAction, UiAction};
use bevy::prelude::*;
use bevy::ui::FocusPolicy;

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
        let tile = spawn_node(
            commands,
            column,
            Node {
                width: px(38),
                height: px(38),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
            None,
        );
        let mut background = ImageNode::new(self.assets.home.button.clone()).with_mode(
            NodeImageMode::Sliced(TextureSlicer {
                border: BorderRect::all(32.0),
                center_scale_mode: SliceScaleMode::Stretch,
                sides_scale_mode: SliceScaleMode::Stretch,
                max_corner_scale: 0.55,
            }),
        );
        background.visual_box = bevy::ui::VisualBox::BorderBox;
        commands.entity(tile).insert((
            Button,
            UiAction::Navigation(NavigationUiAction::ToggleAchievements),
            ButtonTint {
                normal: Color::WHITE,
                hovered: Color::srgb(0.89, 0.85, 1.0),
                pressed: Color::srgb(0.78, 0.73, 0.96),
            },
            background,
        ));
        let icon = commands
            .spawn((
                Node {
                    width: px(40),
                    height: px(40),
                    ..default()
                },
                ImageNode::new(self.assets.achievements.icon.clone()),
                FocusPolicy::Pass,
            ))
            .id();
        commands.entity(tile).add_child(icon);
        add_text(commands, column, "成就", 12.0, TEXT, self.assets);
    }
}
