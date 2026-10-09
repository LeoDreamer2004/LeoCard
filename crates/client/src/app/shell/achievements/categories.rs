use super::super::{PageTransitionElement, UiAction};
use crate::app::shell::AchievementUiAction;
use bevy::picking::Pickable;
use bevy::ui_widgets::Button;

use crate::app::presentation::{MUTED, add_text, spawn_node};
use bevy::prelude::*;
use bevy::ui::RelativeCursorPosition;
use leocard_achievements::achievements_in;

use super::input::category_tint;
use super::page::AchievementsPage;
use super::state::*;

impl AchievementsPage<'_> {
    pub(super) fn render_categories(&self, commands: &mut Commands, parent: Entity) {
        let column = spawn_node(
            commands,
            parent,
            Node {
                width: px(224),
                min_height: px(0),
                flex_shrink: 0.0,
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                row_gap: px(4),
                ..default()
            },
            None,
        );
        commands
            .entity(column)
            .insert((PageTransitionElement::left(1), UiTransform::IDENTITY));
        self.render_scroll_hint(commands, column, false);
        let rail = spawn_node(
            commands,
            column,
            Node {
                width: percent(100),
                min_height: px(0),
                flex_basis: px(0),
                flex_grow: 1.0,
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                overflow: Overflow::scroll_y(),
                ..default()
            },
            None,
        );
        commands.entity(rail).insert((
            AchievementsScroll,
            AchievementCategoryScroll,
            RelativeCursorPosition::default(),
            ScrollPosition(Vec2::new(0.0, self.category_scroll_y)),
        ));
        for (index, (tab, idiom)) in CATEGORIES.into_iter().enumerate() {
            let selected = self.selected == tab;
            let button = commands
                .spawn((
                    Button,
                    UiAction::Achievements(AchievementUiAction::SelectCategory(tab)),
                    Node {
                        width: percent(100),
                        height: px(166),
                        flex_shrink: 0.0,
                        flex_direction: FlexDirection::Column,
                        padding: UiRect::vertical(px(9)),
                        align_items: AlignItems::Center,
                        row_gap: px(4),
                        ..default()
                    },
                    BackgroundColor(Color::NONE),
                ))
                .id();
            commands.entity(rail).add_child(button);
            let art = commands
                .spawn((
                    Node {
                        width: px(112),
                        height: px(112),
                        flex_shrink: 0.0,
                        ..default()
                    },
                    ImageNode::new(self.assets.achievements.emblems[index].clone())
                        .with_color(category_tint(selected, false)),
                    Pickable::IGNORE,
                ))
                .id();
            commands.entity(button).add_child(art);
            commands.entity(button).insert(AchievementCategoryButton {
                selected,
                emblem: art,
            });
            add_text(
                commands,
                button,
                idiom,
                20.0,
                if selected { GOLD } else { MUTED },
                self.assets,
            );
            let track = spawn_node(
                commands,
                button,
                Node {
                    width: px(154),
                    height: px(3),
                    flex_shrink: 0.0,
                    ..default()
                },
                Some(Color::srgba(0.55, 0.50, 0.65, 0.3)),
            );
            spawn_node(
                commands,
                track,
                Node {
                    width: percent({
                        let definitions = achievements_in(tab).collect::<Vec<_>>();
                        if definitions.is_empty() {
                            0.0
                        } else {
                            100.0
                                * definitions
                                    .iter()
                                    .filter(|item| self.progress.achieved(item))
                                    .count() as f32
                                / definitions.len() as f32
                        }
                    }),
                    height: percent(100),
                    ..default()
                },
                Some(if selected {
                    GOLD
                } else {
                    ACCENT.with_alpha(0.4)
                }),
            );
        }
        self.render_scroll_hint(commands, column, true);
    }

    fn render_scroll_hint(&self, commands: &mut Commands, parent: Entity, down: bool) {
        let hint = commands
            .spawn((
                Node {
                    width: px(24),
                    height: px(14),
                    flex_shrink: 0.0,
                    ..default()
                },
                ImageNode::new(self.assets.achievements.scroll_arrow.clone())
                    .with_color(Color::WHITE.with_alpha(0.65)),
                UiTransform::from_rotation(Rot2::degrees(if down { 180.0 } else { 0.0 })),
                Pickable::IGNORE,
            ))
            .id();
        commands.entity(parent).add_child(hint);
    }
}
