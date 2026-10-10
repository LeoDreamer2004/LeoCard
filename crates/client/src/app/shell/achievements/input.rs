use super::state::*;
use bevy::input::mouse::{MouseScrollUnit, MouseWheel};
use bevy::picking::hover::PickingInteraction;
use bevy::prelude::*;
use bevy::ui::RelativeCursorPosition;

pub(crate) fn scroll_achievements(
    mut wheels: MessageReader<MouseWheel>,
    mut scrolls: Query<
        (&RelativeCursorPosition, &mut ScrollPosition, &ComputedNode),
        With<AchievementsScroll>,
    >,
) {
    let delta = wheels
        .read()
        .map(|wheel| match wheel.unit {
            MouseScrollUnit::Line => wheel.y * 40.0,
            MouseScrollUnit::Pixel => wheel.y,
        })
        .sum::<f32>();
    if delta == 0.0 {
        return;
    }
    for (cursor, mut position, node) in &mut scrolls {
        if cursor.cursor_over() {
            let maximum =
                ((node.content_size().y - node.size().y) * node.inverse_scale_factor()).max(0.0);
            position.y = (position.y - delta).clamp(0.0, maximum);
        }
    }
}

pub(crate) fn update_achievement_category_hover(
    buttons: Query<(&PickingInteraction, &AchievementCategoryButton), Changed<PickingInteraction>>,
    mut emblems: Query<&mut ImageNode>,
) {
    for (interaction, category) in &buttons {
        if let Ok(mut image) = emblems.get_mut(category.emblem) {
            image.color =
                category_tint(category.selected, *interaction != PickingInteraction::None);
        }
    }
}

pub(super) fn category_tint(selected: bool, hovered: bool) -> Color {
    if selected || hovered {
        Color::WHITE
    } else {
        Color::srgba(0.72, 0.69, 0.78, 0.65)
    }
}
