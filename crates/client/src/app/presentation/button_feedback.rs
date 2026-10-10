//! 通用按钮的高亮和按压动画，以及交互控件的点击音效。
use super::{BackgroundButtonTint, ButtonTint, UiPress, UiPressTarget};
use crate::app::runtime::UiAssets;
use bevy::picking::hover::PickingInteraction;
use bevy::prelude::*;
use bevy::ui_widgets::Button;
use std::collections::HashSet;

/// The feature animates this control itself.
#[derive(Component, Default)]
pub(crate) struct CustomButtonMotion;

/// The feature supplies its own click sound.
#[derive(Component, Default)]
pub(crate) struct SilentButton;

#[derive(Component)]
pub(crate) struct ButtonHighlight {
    pub overlay: Entity,
    pub arrows: Option<[Entity; 2]>,
}

#[derive(Component)]
pub(crate) struct ButtonArrows {
    pub(crate) left: bool,
    pub(crate) owner: Entity,
    pub(crate) elapsed: f32,
}

pub(crate) fn animate_button_arrows(
    time: Res<Time>,
    interactions: Query<&PickingInteraction>,
    mut images: Query<(&mut ButtonArrows, &mut ImageNode)>,
) {
    for (mut arrow, mut image) in &mut images {
        if !interactions
            .get(arrow.owner)
            .is_ok_and(|state| *state != PickingInteraction::None)
        {
            arrow.elapsed = 0.0;
            continue;
        }
        arrow.elapsed += time.delta_secs();
        let tick = ((arrow.elapsed * 20.0) as usize) % 33;
        let frame = if tick < 16 {
            tick
        } else if tick < 19 {
            15
        } else {
            33 - tick
        } as f32;
        let left = if arrow.left { 8.0 } else { 136.0 };
        let rect = Rect::from_corners(
            Vec2::new(left, frame * 80.0 + 8.0),
            Vec2::new(left + 112.0, frame * 80.0 + 64.0),
        );
        if image.rect != Some(rect) {
            image.rect = Some(rect);
        }
    }
}

pub(crate) fn update_button_highlights(
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
pub(crate) fn update_button_tints(
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

pub(crate) fn play_ui_click_sounds(
    mut presses: MessageReader<UiPress>,
    controls: Query<(), (With<UiPressTarget>, Without<SilentButton>)>,
    assets: Res<UiAssets>,
    mut commands: Commands,
) {
    if assets.audio.button_click_sounds.is_empty() {
        return;
    }
    for press in presses.read() {
        if !controls.contains(press.0) {
            continue;
        }
        let sound = assets.audio.button_click_sounds
            [fastrand::usize(..assets.audio.button_click_sounds.len())]
        .clone();
        commands.spawn((AudioPlayer::new(sound), PlaybackSettings::DESPAWN));
    }
}

#[expect(
    clippy::type_complexity,
    reason = "disjoint filtered Bevy queries separate changed and animated buttons"
)]
pub(crate) fn animate_button_presses(
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
