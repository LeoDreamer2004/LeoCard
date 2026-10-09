use super::{EntryGlow, GameEntry, GlowKind};
use crate::app::presentation::TransitionOpacity;
use crate::app::shell::PageMotion;
use bevy::picking::hover::PickingInteraction;
use bevy::prelude::*;
use bevy::ui::BackgroundGradient;

#[derive(Clone, Copy)]
pub(super) struct EntryMotion {
    hover: f32,
    press: f32,
    ripple: f32,
    was_pressed: bool,
}

impl Default for EntryMotion {
    fn default() -> Self {
        Self {
            hover: 0.0,
            press: 0.0,
            ripple: 1.0,
            was_pressed: false,
        }
    }
}

impl EntryMotion {
    fn advance(&mut self, interaction: PickingInteraction, delta: f32, fresh_press: bool) {
        let hovered = f32::from(interaction != PickingInteraction::None);
        let pressed = interaction == PickingInteraction::Pressed;
        self.hover += (hovered - self.hover) * (1.0 - (-delta * 12.0).exp());
        self.press += (f32::from(pressed) - self.press) * (1.0 - (-delta * 25.0).exp());
        if pressed && !self.was_pressed && fresh_press {
            self.ripple = 0.0;
        }
        self.ripple = (self.ripple + delta / 0.65).min(1.0);
        self.was_pressed = pressed;
    }

    fn art(&self, phase: f32) -> UiTransform {
        UiTransform {
            translation: Val2::px(
                0.0,
                phase.sin() * 1.8 * (1.0 - self.hover) - self.hover * 7.0 + self.press * 4.0,
            ),
            scale: Vec2::splat(1.0 + self.hover * 0.045 - self.press * 0.075),
            rotation: Rot2::degrees(
                (phase * 0.73).sin() * 0.65 * (1.0 - self.hover) - self.hover * 0.6,
            ),
        }
    }

    fn glow(&self, kind: GlowKind, phase: f32) -> (UiTransform, f32) {
        let breath = phase.sin();
        let (scale, strength, rotation) = match kind {
            GlowKind::Aura => (
                1.0 + 0.035 * breath + 0.08 * self.hover,
                0.8 + 0.12 * breath + self.hover * 0.9 + self.press * 0.3,
                0.0,
            ),
            GlowKind::Orbit => (
                1.0 + 0.025 * breath + self.hover * 0.06,
                0.65 + self.hover * 0.75,
                -8.0 + 2.0 * (phase * 0.5).sin(),
            ),
            GlowKind::Ripple => (0.65 + self.ripple * 0.65, (1.0 - self.ripple).powi(2), 0.0),
            GlowKind::Underline => (1.0 + self.hover * 0.2, 0.75 + self.hover * 0.6, 0.0),
        };
        (
            UiTransform {
                scale: Vec2::splat(scale),
                rotation: Rot2::degrees(rotation),
                ..default()
            },
            strength,
        )
    }
}

pub(super) fn animate_entries(
    time: Res<Time>,
    mouse: Res<ButtonInput<MouseButton>>,
    page: Res<PageMotion>,
    entries: Query<(&PickingInteraction, &GameEntry)>,
    mut states: Local<[EntryMotion; 5]>,
    mut transforms: Query<&mut UiTransform>,
    mut glows: Query<(&EntryGlow, &TransitionOpacity, &mut BackgroundGradient)>,
) {
    if entries.is_empty() {
        *states = default();
        return;
    }
    let transitioning = page.active();
    for (interaction, entry) in &entries {
        let state = &mut states[entry.index];
        state.advance(
            if transitioning {
                PickingInteraction::None
            } else {
                *interaction
            },
            time.delta_secs(),
            !mouse.pressed(MouseButton::Left) || mouse.just_pressed(MouseButton::Left),
        );
        let phase = time.elapsed_secs() * 0.65 + entry.index as f32 * 1.37;
        if let Ok(mut transform) = transforms.get_mut(entry.art) {
            *transform = state.art(phase);
        }
        for &entity in &entry.glows {
            if let Ok((glow, opacity, mut gradient)) = glows.get_mut(entity) {
                let (transform, strength) = state.glow(glow.kind, phase);
                glow.animate(&mut gradient, strength * opacity.factor());
                if let Ok(mut current) = transforms.get_mut(entity) {
                    *current = transform;
                }
            }
        }
    }
}
