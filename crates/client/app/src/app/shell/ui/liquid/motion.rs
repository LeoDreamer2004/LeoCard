use super::material::MAX_RIPPLES;
use super::{LiquidKind, LiquidMaterial, LiquidSurface, LiquidUvRect};
use crate::app::shell::PageMotion;
use bevy::picking::events::PointerPress;
use bevy::picking::hover::PickingInteraction;
use bevy::picking::pointer::PointerButton;
use bevy::prelude::*;
use bevy::ui::{InteractionDisabled, RelativeCursorPosition};
use leocard_ui::TransitionOpacity;

#[derive(Component)]
pub(super) struct LiquidMotion {
    material: Handle<LiquidMaterial>,
    pointer: Vec2,
    velocity: Vec2,
    hover: f32,
    press: f32,
    ripples: Vec<Vec4>,
    sweep: f32,
}

pub(super) fn initialize_liquid(
    mut commands: Commands,
    mut materials: ResMut<Assets<LiquidMaterial>>,
    surfaces: Query<(Entity, &LiquidSurface, Option<&LiquidUvRect>), Without<LiquidMotion>>,
) {
    for (entity, surface, rect) in &surfaces {
        let material = materials.add(LiquidMaterial {
            tint: surface.tint.to_linear().to_vec4(),
            animation: Vec4::W,
            pointer: Vec2::splat(0.5),
            params: Vec4::new(
                match surface.kind {
                    LiquidKind::Card => 0.0,
                    LiquidKind::Accent => 1.0,
                    LiquidKind::JoinSweep => 2.0,
                },
                surface.phase,
                0.0,
                0.0,
            ),
            uv_rect: rect.map_or(Vec4::new(0.0, 0.0, 1.0, 1.0), |rect| rect.0),
            ripples: [Vec4::ZERO; MAX_RIPPLES],
            texture: surface.texture.clone(),
        });
        commands.entity(entity).insert((
            MaterialNode(material.clone()),
            LiquidMotion {
                material,
                pointer: Vec2::splat(0.5),
                velocity: Vec2::ZERO,
                hover: 0.0,
                press: 0.0,
                ripples: Vec::new(),
                sweep: 0.0,
            },
        ));
    }
}

pub(super) fn capture_ripples(
    mut presses: MessageReader<PointerPress>,
    page: Res<PageMotion>,
    controls: Query<&Children, Without<InteractionDisabled>>,
    mut surfaces: Query<(&LiquidSurface, &mut LiquidMotion)>,
) {
    for press in presses.read() {
        if page.active()
            || !matches!(
                press.button,
                PointerButton::Primary | PointerButton::Secondary
            )
        {
            continue;
        }
        let (Ok(children), Some(position)) = (controls.get(press.entity), press.hit.position)
        else {
            continue;
        };
        for child in children.iter() {
            if let Ok((surface, mut state)) = surfaces.get_mut(child)
                && surface.owner == press.entity
                && matches!(surface.kind, LiquidKind::Card)
                && state.ripples.len() < MAX_RIPPLES
            {
                // UI hit positions are centered, and the ripple surface fills the card.
                let uv = (position.truncate() + Vec2::splat(0.5)).clamp(Vec2::ZERO, Vec2::ONE);
                state.ripples.push(uv.extend(0.0).extend(0.0));
            }
        }
    }
}

pub(super) fn animate_liquid(
    time: Res<Time>,
    page: Res<PageMotion>,
    controls: Query<(Option<&PickingInteraction>, &RelativeCursorPosition)>,
    positions: Query<&RelativeCursorPosition>,
    mut surfaces: Query<(&LiquidSurface, &TransitionOpacity, &mut LiquidMotion)>,
    mut materials: ResMut<Assets<LiquidMaterial>>,
) {
    let dt = time.delta_secs().min(0.05);
    for (surface, opacity, mut state) in &mut surfaces {
        state.ripples.retain_mut(|ripple| {
            ripple.z += dt;
            ripple.z < 1.95
        });
        if matches!(surface.kind, LiquidKind::Accent) {
            if let Some(mut material) = materials.get_mut(&state.material) {
                material.animation.w = opacity.factor();
            }
            continue;
        }
        let Ok((interaction, cursor)) = controls.get(surface.owner) else {
            continue;
        };
        let interaction = interaction.copied().unwrap_or(PickingInteraction::None);
        let over_surface = surface.hit == surface.owner
            || positions
                .get(surface.hit)
                .is_ok_and(|cursor| cursor.cursor_over);
        let hovered = interaction != PickingInteraction::None && over_surface && !page.active();
        let pressed = interaction == PickingInteraction::Pressed;
        // Bevy's relative position is centered: (-0.5, -0.5) is the top-left.
        // Texture coordinates instead start at (0, 0), regardless of UiScale/DPI.
        let pointer = cursor
            .normalized
            .map(|point| point + Vec2::splat(0.5))
            .unwrap_or(Vec2::splat(0.5))
            .clamp(Vec2::ZERO, Vec2::ONE);
        state.sweep = if hovered { state.sweep + dt } else { 0.0 };
        state.hover += (f32::from(hovered) - state.hover) * (1.0 - (-dt * 9.0).exp());
        state.press += (f32::from(pressed && hovered) - state.press) * (1.0 - (-dt * 20.0).exp());
        let target = if hovered { pointer } else { Vec2::splat(0.5) };
        // Two small steps keep the pointer's damped spring stable on slower phones.
        for _ in 0..2 {
            let acceleration = (target - state.pointer) * 115.0 - state.velocity * 18.0;
            state.velocity += acceleration * dt * 0.5;
            let velocity = state.velocity;
            state.pointer += velocity * dt * 0.5;
        }
        if let Some(mut material) = materials.get_mut(&state.material) {
            material.animation.x = time.elapsed_secs();
            material.animation.y = state.hover;
            material.animation.w = opacity.factor();
            material.set_ripples(&state.ripples);
            material.pointer = state.pointer;
            material.params.z = if !matches!(surface.kind, LiquidKind::Card) {
                f32::from(hovered)
            } else {
                state.press
            };
            material.params.w = state.sweep;
        }
    }
}
