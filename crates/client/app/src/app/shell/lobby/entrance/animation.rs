use super::material::EntranceVeil;
use super::state::LobbyEntranceState;
use super::view::{
    EntranceHeading, EntranceOverlay, EntranceParticle, EntranceParty, EntranceText,
    LobbySeatedAvatar,
};
use crate::app::presentation::ease_out_cubic;
use bevy::prelude::*;

pub(super) fn animate_heading(
    state: Res<LobbyEntranceState>,
    mut headings: Query<&mut UiTransform, With<EntranceHeading>>,
    mut text: Query<(&mut TextColor, &mut TextShadow), With<EntranceText>>,
) {
    let Some(active) = state.active else { return };
    let time = active.elapsed;
    let enter = ease_out_cubic((time / 0.42).clamp(0.0, 1.0));
    let fade = fade(time);
    for mut transform in &mut headings {
        transform.translation = Val2::px(0.0, -180.0 * (1.0 - enter));
        transform.scale = Vec2::splat(0.96 + 0.04 * enter);
    }
    for (mut color, mut shadow) in &mut text {
        color.0.set_alpha(enter * fade);
        shadow.color.set_alpha(0.8 * enter * fade);
    }
}

pub(super) fn animate_parties(
    state: Res<LobbyEntranceState>,
    mut parties: Query<(&EntranceParty, &mut UiTransform, &mut ImageNode)>,
) {
    let Some(active) = state.active else { return };
    let time = active.elapsed;
    let enter = ease_out_cubic((time / 0.42).clamp(0.0, 1.0));
    let fade = fade(time);
    for (party, mut transform, mut image) in &mut parties {
        let pulse = (time * 9.0).sin() * (1.0 - time / 2.4).max(0.0);
        transform.rotation = Rot2::radians(party.0 * pulse * 0.12);
        transform.scale = Vec2::splat(1.0 + pulse * 0.08);
        image.color.set_alpha(enter * fade);
    }
}

pub(super) fn animate_confetti(
    state: Res<LobbyEntranceState>,
    mut particles: Query<(&EntranceParticle, &mut UiTransform, &mut BackgroundColor)>,
) {
    let Some(active) = state.active else { return };
    let time = active.elapsed;
    let fade = fade(time);
    for (particle, mut transform, mut color) in &mut particles {
        let age = (time - particle.delay).max(0.0);
        let alpha = if time < particle.delay {
            0.0
        } else {
            (1.0 - age / 1.9).clamp(0.0, 1.0) * fade
        };
        let point = particle.origin + particle.velocity * age + Vec2::Y * (145.0 * age * age);
        transform.translation = Val2::px(point.x, point.y);
        transform.rotation = Rot2::radians(particle.spin * age);
        transform.scale = Vec2::new(0.3 + (age * 8.0 + particle.spin).cos().abs() * 0.7, 1.0);
        color.0 = particle.color.with_alpha(alpha);
    }
}

pub(super) fn sync_spotlight(
    state: Res<LobbyEntranceState>,
    avatars: Query<(
        &LobbySeatedAvatar,
        &ComputedNode,
        &UiGlobalTransform,
        &InheritedVisibility,
    )>,
    overlays: Query<&EntranceOverlay>,
    mut materials: ResMut<Assets<EntranceVeil>>,
) {
    let Some(active) = state.active else { return };
    let entrance = ease_out_cubic((active.elapsed / 0.2).clamp(0.0, 1.0));
    let fade = fade(active.elapsed);
    let mut circle = Vec4::new(-10_000.0, -10_000.0, 0.0, 0.72 * entrance * fade);
    if let Some((_, node, transform, _)) = avatars.iter().find(|(avatar, node, _, visible)| {
        avatar.0 == active.player && visible.get() && node.size().min_element() > 1.0
    }) {
        let (scale, _, center) = transform.to_scale_angle_translation();
        circle.x = center.x;
        circle.y = center.y;
        circle.z = node.size().min_element() * scale.abs().min_element() * 0.5;
    }
    for overlay in &overlays {
        if let Some(mut material) = materials.get_mut(&overlay.veil) {
            material.spotlight = circle;
        }
    }
}

fn fade(elapsed: f32) -> f32 {
    1.0 - ease_out_cubic(((elapsed - 2.0) / 0.4).clamp(0.0, 1.0))
}
