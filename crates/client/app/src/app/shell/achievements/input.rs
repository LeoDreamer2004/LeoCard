use super::state::*;
use bevy::picking::hover::PickingInteraction;
use bevy::prelude::*;

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
