use super::super::{PageTransitionElement, add_cozy_panel};
use crate::app::presentation::{MUTED, TEXT, add_text, spawn_node};
use bevy::prelude::*;
use bevy::ui::{FocusPolicy, RelativeCursorPosition, VisualBox};
use leocard_client::{AchievementDefinition, achievements_in};

use super::page::AchievementsPage;
use super::state::*;

impl AchievementsPage<'_> {
    pub(super) fn render_entries(&self, commands: &mut Commands, parent: Entity) {
        let panel = add_cozy_panel(
            commands,
            parent,
            Node {
                min_width: px(0),
                min_height: px(0),
                flex_grow: 1.0,
                padding: UiRect::all(px(22)),
                flex_direction: FlexDirection::Column,
                row_gap: px(14),
                ..default()
            },
            self.assets,
        );
        let idiom = CATEGORIES
            .into_iter()
            .find(|(tab, _)| *tab == self.selected)
            .map(|(_, idiom)| idiom)
            .unwrap();
        commands
            .entity(panel)
            .insert((PageTransitionElement::right(2), UiTransform::IDENTITY));
        add_text(commands, panel, idiom, 25.0, GOLD, self.assets);
        spawn_node(
            commands,
            panel,
            Node {
                width: percent(100),
                height: px(2),
                ..default()
            },
            Some(ACCENT.with_alpha(0.65)),
        );
        let scroll = spawn_node(
            commands,
            panel,
            Node {
                width: percent(100),
                min_height: px(0),
                flex_grow: 1.0,
                flex_direction: FlexDirection::Column,
                row_gap: px(12),
                overflow: Overflow::scroll_y(),
                ..default()
            },
            None,
        );
        commands.entity(scroll).insert((
            AchievementsScroll,
            RelativeCursorPosition::default(),
            ScrollPosition(Vec2::ZERO),
        ));
        let mut definitions = achievements_in(self.selected).peekable();
        if definitions.peek().is_none() {
            add_text(
                commands,
                scroll,
                "此分类的成就内容待定",
                18.0,
                MUTED,
                self.assets,
            );
        }
        for achievement in definitions {
            self.render_entry(commands, scroll, achievement);
        }
    }

    fn render_entry(
        &self,
        commands: &mut Commands,
        parent: Entity,
        achievement: &AchievementDefinition,
    ) {
        let row = commands
            .spawn((
                Node {
                    width: percent(100),
                    min_height: px(116),
                    flex_shrink: 0.0,
                    padding: UiRect::axes(px(20), px(14)),
                    align_items: AlignItems::Center,
                    column_gap: px(18),
                    ..default()
                },
                card_texture(self.assets.home.game_card.clone()),
            ))
            .id();
        commands.entity(parent).add_child(row);
        let trophy = commands
            .spawn((
                Node {
                    width: px(76),
                    height: px(76),
                    flex_shrink: 0.0,
                    ..default()
                },
                ImageNode::new(
                    self.assets.achievements.medals[achievement.tier.medal_index()].clone(),
                ),
                FocusPolicy::Pass,
            ))
            .id();
        commands.entity(row).add_child(trophy);
        let text = spawn_node(
            commands,
            row,
            Node {
                min_width: px(0),
                flex_grow: 1.0,
                flex_direction: FlexDirection::Column,
                row_gap: px(6),
                ..default()
            },
            None,
        );
        add_text(commands, text, achievement.title, 23.0, TEXT, self.assets);
        add_text(
            commands,
            text,
            achievement.description,
            15.0,
            MUTED,
            self.assets,
        );
        if self.progress.achieved(achievement) {
            let status = spawn_node(
                commands,
                row,
                Node {
                    min_width: px(118),
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::FlexEnd,
                    row_gap: px(7),
                    ..default()
                },
                None,
            );
            add_text(
                commands,
                status,
                "已达成",
                17.0,
                Color::srgb(0.84, 0.91, 0.74),
                self.assets,
            );
            if let Some(date) = self.progress.date_label(achievement) {
                add_text(commands, status, date, 12.0, MUTED, self.assets);
            }
        }
    }
}

fn card_texture(texture: Handle<Image>) -> ImageNode {
    let mut image = ImageNode::new(texture).with_mode(NodeImageMode::Sliced(TextureSlicer {
        border: BorderRect::all(16.0),
        center_scale_mode: SliceScaleMode::Stretch,
        sides_scale_mode: SliceScaleMode::Stretch,
        max_corner_scale: 1.0,
    }));
    image.visual_box = VisualBox::BorderBox;
    image
}
