use super::{
    UNO_FLYING_CARD_HEIGHT, UNO_FLYING_CARD_WIDTH, UNO_PALETTE_EFFECT_DURATION,
    UNO_REVERSE_EFFECT_DURATION, UnoFlipCard, UnoFlipOverlay, UnoFlyingCard, UnoPaletteColorRing,
    UnoPaletteEffect, UnoPaletteMaterial, UnoPaletteParticle, UnoPaletteSelectedSector,
    UnoReverseArrow, quadratic_bezier,
};
use crate::app::presentation::ease_out_cubic;
use bevy::prelude::*;

pub(crate) fn animate_uno_flying_cards(
    time: Res<Time>,
    mut commands: Commands,
    mut cards: Query<(
        Entity,
        &mut UnoFlyingCard,
        &mut Node,
        &mut UiTransform,
        &mut ImageNode,
    )>,
) {
    for (entity, mut flight, mut node, mut transform, mut image) in &mut cards {
        flight.elapsed += time.delta_secs();
        let local = flight.elapsed - flight.delay;
        if local < 0.0 {
            continue;
        }
        let progress = (local / flight.duration).clamp(0.0, 1.0);
        let (position, motion) = if flight.draw_animation {
            if progress < 0.22 {
                let t = smoothstep(progress / 0.22);
                (flight.start.lerp(flight.staging, t), t * 0.12)
            } else if progress < 0.43 {
                (flight.staging, 0.12)
            } else {
                let t = smoothstep((progress - 0.43) / 0.57);
                (
                    quadratic_bezier(flight.staging, flight.control, flight.target, t),
                    0.12 + t * 0.88,
                )
            }
        } else {
            let t = 1.0 - (1.0 - progress).powi(3);
            (
                quadratic_bezier(flight.start, flight.control, flight.target, t),
                t,
            )
        };
        node.left = px(position.x - UNO_FLYING_CARD_WIDTH * 0.5);
        node.top = px(position.y - UNO_FLYING_CARD_HEIGHT * 0.5);
        transform.rotation =
            Rot2::degrees(flight.start_angle + (flight.end_angle - flight.start_angle) * motion);
        transform.scale = Vec2::splat(uno_flying_card_scale(flight.draw_animation, progress));
        let fade_in = (local / 0.08).clamp(0.0, 1.0);
        let fade_out = if flight.draw_animation {
            ((1.0 - progress) / 0.10).clamp(0.0, 1.0)
        } else {
            1.0
        };
        image.color = Color::WHITE.with_alpha(fade_in * fade_out);
        if progress >= 1.0 {
            commands.entity(entity).despawn();
        }
    }
}

pub(crate) fn animate_uno_flip_effects(
    mut commands: Commands,
    time: Res<Time>,
    mut overlays: Query<(Entity, &mut UnoFlipOverlay, &mut BackgroundColor)>,
    mut cards: Query<(Entity, &mut UnoFlipCard, &mut UiTransform, &mut ImageNode)>,
) {
    let delta = time.delta_secs();
    for (entity, mut effect, mut background) in &mut overlays {
        effect.elapsed += delta;
        let progress = (effect.elapsed / 2.0).clamp(0.0, 1.0);
        let alpha = (std::f32::consts::PI * progress).sin().powf(1.35) * 0.34;
        background.0.set_alpha(alpha);
        if progress >= 1.0 {
            commands.entity(entity).despawn();
        }
    }
    for (entity, mut card, mut transform, mut image) in &mut cards {
        card.elapsed += delta;
        let progress = ((card.elapsed - card.delay) / 1.18).clamp(0.0, 1.0);
        *transform = card.base_transform;
        if progress <= 0.0 {
            continue;
        }
        if progress >= 0.5 && !card.swapped {
            image.image = card.new_face.clone();
            card.swapped = true;
        } else if progress < 0.5 && card.swapped {
            image.image = card.old_face.clone();
            card.swapped = false;
        }
        let edge = (std::f32::consts::PI * progress).cos().abs().max(0.028);
        let lift = (std::f32::consts::PI * progress).sin();
        let scale_y = if card.pile {
            1.0 + lift * 0.09
        } else {
            1.0 + lift * 0.13
        };
        transform.scale = card.base_transform.scale * Vec2::new(edge, scale_y);
        transform.rotation = card.base_transform.rotation
            * Rot2::degrees((progress - 0.5) * if card.pile { 3.0 } else { 8.0 });
        transform.translation = card
            .base_transform
            .translation
            .try_add(Val2::px(0.0, -lift * if card.pile { 28.0 } else { 14.0 }))
            .unwrap_or(card.base_transform.translation);
        if progress >= 1.0 {
            image.image = card.new_face.clone();
            *transform = card.base_transform;
            commands.entity(entity).remove::<UnoFlipCard>();
        }
    }
}

pub(crate) fn uno_flying_card_scale(draw_animation: bool, progress: f32) -> f32 {
    let progress = progress.clamp(0.0, 1.0);
    if !draw_animation && progress >= 1.0 {
        return 1.0;
    }
    if draw_animation {
        0.76 + (progress * std::f32::consts::PI).sin() * 0.10
    } else {
        let settle = smoothstep(progress);
        0.76 + settle * 0.24 + (progress * std::f32::consts::PI).sin() * 0.08
    }
}

pub(crate) fn animate_uno_palette_effects(
    time: Res<Time>,
    mut commands: Commands,
    mut materials: ResMut<Assets<UnoPaletteMaterial>>,
    mut palettes: Query<(
        Entity,
        &mut UnoPaletteEffect,
        &mut UiTransform,
        &MaterialNode<UnoPaletteMaterial>,
        &mut BoxShadow,
    )>,
) {
    for (entity, mut effect, mut transform, material_node, mut shadow) in &mut palettes {
        effect.elapsed += time.delta_secs();
        let progress = (effect.elapsed / UNO_PALETTE_EFFECT_DURATION).clamp(0.0, 1.0);
        let entry = ease_out_back((progress / 0.17).clamp(0.0, 1.0));
        let fade = smoothstep(((1.0 - progress) / 0.15).clamp(0.0, 1.0));
        transform.scale = Vec2::splat(0.72 + entry * 0.28);
        transform.rotation = Rot2::degrees((1.0 - entry) * -16.0);
        let opacity = entry.clamp(0.0, 1.0) * fade;
        if let Some(mut material) = materials.get_mut(&material_node.0) {
            material.params.z = opacity;
        }
        if let Some(style) = shadow.0.first_mut() {
            style.color = Color::BLACK.with_alpha(opacity * 0.50);
            style.blur_radius = px(9.0 + entry * 5.0);
        }
        if progress >= 1.0 {
            commands.entity(entity).despawn();
        }
    }
}

pub(crate) fn animate_uno_palette_selected_sectors(
    time: Res<Time>,
    mut materials: ResMut<Assets<UnoPaletteMaterial>>,
    mut sectors: Query<(
        &mut UnoPaletteSelectedSector,
        &mut UiTransform,
        &MaterialNode<UnoPaletteMaterial>,
    )>,
) {
    for (mut effect, mut transform, material_node) in &mut sectors {
        effect.elapsed += time.delta_secs();
        let progress = (effect.elapsed / UNO_PALETTE_EFFECT_DURATION).clamp(0.0, 1.0);
        let entry = smoothstep((progress / 0.16).clamp(0.0, 1.0));
        let fade = smoothstep(((1.0 - progress) / 0.15).clamp(0.0, 1.0));
        transform.scale = Vec2::splat(uno_palette_selected_scale(progress));
        if let Some(mut material) = materials.get_mut(&material_node.0) {
            material.params.z = entry * fade;
        }
    }
}

pub(crate) fn animate_uno_palette_color_rings(
    time: Res<Time>,
    mut rings: Query<(&mut UnoPaletteColorRing, &mut BorderColor, &mut UiTransform)>,
) {
    for (mut ring, mut border, mut transform) in &mut rings {
        ring.elapsed += time.delta_secs();
        let progress = ((ring.elapsed - ring.delay) / UNO_PALETTE_EFFECT_DURATION).clamp(0.0, 1.0);
        let burst = ((progress - 0.34) / 0.43).clamp(0.0, 1.0);
        let motion = smoothstep(burst);
        let opacity = (burst * std::f32::consts::PI).sin().max(0.0).powf(0.72);
        transform.scale =
            Vec2::splat(ring.start_scale + (ring.end_scale - ring.start_scale) * motion);
        border.set_all(ring.color.with_alpha(opacity * ring.max_alpha));
    }
}

pub(crate) fn animate_uno_palette_particles(
    time: Res<Time>,
    mut particles: Query<(
        &mut UnoPaletteParticle,
        &mut Node,
        &mut BackgroundColor,
        &mut UiTransform,
    )>,
) {
    for (mut particle, mut node, mut background, mut transform) in &mut particles {
        particle.elapsed += time.delta_secs();
        let local = ((particle.elapsed - particle.delay) / 0.82).clamp(0.0, 1.0);
        let motion = ease_out_cubic(local);
        let opacity = (local * std::f32::consts::PI).sin().max(0.0).powf(0.65);
        let position = particle.origin + particle.direction * motion;
        node.left = px(position.x - particle.size.x * 0.5);
        node.top = px(position.y - particle.size.y * 0.5);
        background.0 = particle.color.with_alpha(opacity * 0.92);
        transform.scale = Vec2::splat(0.25 + opacity * 0.95);
        transform.rotation = Rot2::degrees(particle.rotation + motion * 115.0);
    }
}

pub(crate) fn uno_palette_selected_scale(progress: f32) -> f32 {
    let progress = progress.clamp(0.0, 1.0);
    if progress <= 0.18 {
        1.0
    } else if progress <= 0.62 {
        1.0 + ease_out_back((progress - 0.18) / 0.44) * 0.32
    } else {
        1.32 - smoothstep((progress - 0.62) / 0.38) * 0.07
    }
}

pub(crate) fn animate_uno_reverse_effects(
    time: Res<Time>,
    mut commands: Commands,
    mut arrows: Query<(
        Entity,
        &mut UnoReverseArrow,
        &mut BackgroundColor,
        &mut BoxShadow,
        &mut UiTransform,
    )>,
) {
    for (entity, mut arrow, mut background, mut shadow, mut transform) in &mut arrows {
        arrow.elapsed += time.delta_secs();
        let local = arrow.elapsed - arrow.delay;
        if local < 0.0 {
            continue;
        }
        let entry = ease_out_back((local / 0.16).clamp(0.0, 1.0));
        let fade =
            smoothstep(((UNO_REVERSE_EFFECT_DURATION - arrow.elapsed) / 0.30).clamp(0.0, 1.0));
        let opacity = entry.clamp(0.0, 1.0) * fade;
        background.0 = arrow.color.with_alpha(opacity * arrow.max_alpha);
        if let Some(style) = shadow.0.first_mut() {
            style.color = Color::BLACK.with_alpha(opacity * arrow.shadow_alpha);
        }
        transform.scale = Vec2::splat(0.68 + entry * 0.32);
        if arrow.elapsed >= UNO_REVERSE_EFFECT_DURATION {
            commands.entity(entity).despawn();
        }
    }
}

fn ease_out_back(value: f32) -> f32 {
    let value = value.clamp(0.0, 1.0);
    let shifted = value - 1.0;
    1.0 + 2.70158 * shifted.powi(3) + 1.70158 * shifted.powi(2)
}

fn smoothstep(value: f32) -> f32 {
    let value = value.clamp(0.0, 1.0);
    value * value * (3.0 - 2.0 * value)
}
