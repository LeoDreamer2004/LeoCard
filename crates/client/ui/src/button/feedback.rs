use super::{BackgroundButtonTint, ButtonHighlight, ButtonTint, CustomButtonMotion};
use bevy::picking::hover::PickingInteraction;
use bevy::prelude::*;
use bevy::ui_widgets::Button;
use std::collections::HashSet;

pub fn update_button_highlights(
    buttons: Query<(&PickingInteraction, &ButtonHighlight), Changed<PickingInteraction>>,
    mut highlights: Query<&mut Visibility>,
) {
    for (interaction, highlight) in &buttons {
        let visibility = if *interaction != PickingInteraction::None {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
        if let Ok(mut overlay) = highlights.get_mut(highlight.overlay) {
            *overlay = visibility;
        }
        if let Some(arrows) = highlight.arrows {
            for arrow in arrows {
                if let Ok(mut arrow) = highlights.get_mut(arrow) {
                    *arrow = visibility;
                }
            }
        }
    }
}

#[expect(
    clippy::type_complexity,
    reason = "separate filtered Bevy queries update image and background buttons"
)]
pub fn update_button_tints(
    mut image_buttons: Query<
        (&PickingInteraction, &ButtonTint, &mut ImageNode),
        (Changed<PickingInteraction>, Without<BackgroundButtonTint>),
    >,
    mut background_buttons: Query<
        (&PickingInteraction, &ButtonTint, &mut BackgroundColor),
        (Changed<PickingInteraction>, With<BackgroundButtonTint>),
    >,
) {
    for (interaction, tint, mut image) in &mut image_buttons {
        image.color = match interaction {
            PickingInteraction::None => tint.normal,
            PickingInteraction::Hovered => tint.hovered,
            PickingInteraction::Pressed => tint.pressed,
        };
    }
    for (interaction, tint, mut background) in &mut background_buttons {
        background.0 = match interaction {
            PickingInteraction::None => tint.normal,
            PickingInteraction::Hovered => tint.hovered,
            PickingInteraction::Pressed => tint.pressed,
        };
    }
}

#[expect(
    clippy::type_complexity,
    reason = "disjoint filtered Bevy queries separate changed and animated buttons"
)]
pub fn animate_button_presses(
    time: Res<Time>,
    changed: Query<
        (Entity, &PickingInteraction),
        (
            Changed<PickingInteraction>,
            With<Button>,
            Without<CustomButtonMotion>,
        ),
    >,
    mut buttons: Query<
        (&PickingInteraction, &mut UiTransform),
        (With<Button>, Without<CustomButtonMotion>),
    >,
    mut active: Local<HashSet<Entity>>,
) {
    active.extend(changed.iter().map(|(entity, _)| entity));
    if active.is_empty() {
        return;
    }
    let response = 1.0 - (-time.delta_secs() * 28.0).exp();
    let entities = active.iter().copied().collect::<Vec<_>>();
    for entity in entities {
        let Ok((interaction, mut transform)) = buttons.get_mut(entity) else {
            active.remove(&entity);
            continue;
        };
        let (target_scale, target_y) = match interaction {
            PickingInteraction::Pressed => (0.97, 1.5),
            PickingInteraction::Hovered => (1.015, 0.0),
            PickingInteraction::None => (1.0, 0.0),
        };
        let scale = transform.scale.x + (target_scale - transform.scale.x) * response;
        let current_y = match transform.translation.y {
            Val::Px(value) => value,
            _ => 0.0,
        };
        let y = current_y + (target_y - current_y) * response;
        if (scale - target_scale).abs() < 0.000_5 && (y - target_y).abs() < 0.01 {
            if transform.scale.x != target_scale || current_y != target_y {
                transform.scale = Vec2::splat(target_scale);
                transform.translation.y = px(target_y);
            }
            active.remove(&entity);
        } else {
            transform.scale = Vec2::splat(scale);
            transform.translation.y = px(y);
        }
    }
}
