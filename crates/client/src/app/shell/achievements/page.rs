use super::super::{NavigationUiAction, PageTransitionElement, UiAction};
use crate::app::presentation::{TEXT, add_text, spawn_node};
use crate::app::runtime::UiAssets;
use bevy::prelude::*;
use bevy::ui::FocusPolicy;
use leocard_achievements::AchievementCategory;
use leocard_client::PlayerAchievements;

pub(crate) struct AchievementsPage<'a> {
    pub(super) selected: AchievementCategory,
    pub(super) category_scroll_y: f32,
    pub(super) progress: &'a PlayerAchievements,
    pub(super) assets: &'a UiAssets,
}

impl<'a> AchievementsPage<'a> {
    pub(crate) fn new(
        selected: AchievementCategory,
        category_scroll_y: f32,
        progress: &'a PlayerAchievements,
        assets: &'a UiAssets,
    ) -> Self {
        Self {
            selected,
            category_scroll_y,
            progress,
            assets,
        }
    }

    pub(crate) fn render(&self, commands: &mut Commands, root: Entity) {
        let canvas = spawn_node(
            commands,
            root,
            Node {
                width: percent(100),
                min_height: px(0),
                flex_basis: px(0),
                flex_grow: 1.0,
                justify_content: JustifyContent::Center,
                ..default()
            },
            None,
        );
        let page = spawn_node(
            commands,
            canvas,
            Node {
                width: percent(100),
                height: percent(100),
                min_height: px(0),
                max_width: px(1360),
                flex_grow: 1.0,
                padding: UiRect::axes(px(24), px(18)),
                flex_direction: FlexDirection::Column,
                row_gap: px(16),
                ..default()
            },
            None,
        );
        self.render_heading(commands, page);
        let body = spawn_node(
            commands,
            page,
            Node {
                width: percent(100),
                min_height: px(0),
                flex_basis: px(0),
                flex_grow: 1.0,
                flex_direction: FlexDirection::Row,
                column_gap: px(18),
                ..default()
            },
            None,
        );
        self.render_categories(commands, body);
        self.render_entries(commands, body);
    }

    fn render_heading(&self, commands: &mut Commands, parent: Entity) {
        let heading = spawn_node(
            commands,
            parent,
            Node {
                width: percent(100),
                flex_shrink: 0.0,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::SpaceBetween,
                ..default()
            },
            None,
        );
        let title = spawn_node(
            commands,
            heading,
            Node {
                align_items: AlignItems::Center,
                column_gap: px(12),
                ..default()
            },
            None,
        );
        commands
            .entity(heading)
            .insert((PageTransitionElement::left(0), UiTransform::IDENTITY));
        commands.entity(title).insert((
            Button,
            UiAction::Navigation(NavigationUiAction::ToggleAchievements),
        ));
        let arrow = commands
            .spawn((
                Node {
                    width: px(24),
                    height: px(14),
                    margin: UiRect::horizontal(px(6)),
                    ..default()
                },
                ImageNode::new(self.assets.achievements.scroll_arrow.clone()),
                UiTransform::from_rotation(Rot2::degrees(-90.0)),
                FocusPolicy::Pass,
            ))
            .id();
        commands.entity(title).add_child(arrow);
        add_text(commands, title, "成就图鉴", 31.0, TEXT, self.assets);
        let trophies = spawn_node(
            commands,
            heading,
            Node {
                align_items: AlignItems::Center,
                column_gap: px(24),
                flex_shrink: 0.0,
                ..default()
            },
            None,
        );
        let counts = self.progress.counts().by_tier();
        for (index, count) in counts.into_iter().enumerate() {
            let counter = spawn_node(
                commands,
                trophies,
                Node {
                    align_items: AlignItems::Center,
                    column_gap: px(6),
                    ..default()
                },
                None,
            );
            let trophy = commands
                .spawn((
                    Node {
                        width: px(42),
                        height: px(42),
                        flex_shrink: 0.0,
                        ..default()
                    },
                    ImageNode::new(self.assets.achievements.medals[index].clone()),
                    FocusPolicy::Pass,
                ))
                .id();
            commands.entity(counter).add_child(trophy);
            add_text(
                commands,
                counter,
                count.to_string(),
                22.0,
                TEXT,
                self.assets,
            );
        }
    }
}
