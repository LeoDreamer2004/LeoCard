//! Slow independent motion, sampled from global time rather than page lifetimes.

use super::{BackgroundAssets, BackgroundMaterial};
use bevy::prelude::*;
use std::f32::consts;

#[derive(Clone, Copy)]
pub(super) struct SuitPlacement {
    pub atlas: usize,
    pub center: Vec2,
    pub size: f32,
    pub tilt: f32,
    pub opacity: f32,
}

// Large shapes stay near the edges; smaller ones leave the content area quiet.
pub(super) const SUITS: [SuitPlacement; 8] = [
    SuitPlacement {
        atlas: 0,
        center: Vec2::new(70.0, 1.0),
        size: 42.0,
        tilt: -0.24,
        opacity: 0.15,
    },
    SuitPlacement {
        atlas: 1,
        center: Vec2::new(3.0, 52.0),
        size: 28.0,
        tilt: 0.20,
        opacity: 0.13,
    },
    SuitPlacement {
        atlas: 2,
        center: Vec2::new(43.0, 100.0),
        size: 33.0,
        tilt: -0.16,
        opacity: 0.12,
    },
    SuitPlacement {
        atlas: 3,
        center: Vec2::new(98.0, 72.0),
        size: 31.0,
        tilt: 0.26,
        opacity: 0.14,
    },
    SuitPlacement {
        atlas: 0,
        center: Vec2::new(-3.0, 94.0),
        size: 41.0,
        tilt: -0.32,
        opacity: 0.13,
    },
    SuitPlacement {
        atlas: 1,
        center: Vec2::new(36.0, 16.0),
        size: 12.0,
        tilt: 0.14,
        opacity: 0.09,
    },
    SuitPlacement {
        atlas: 2,
        center: Vec2::new(90.0, 24.0),
        size: 14.0,
        tilt: -0.12,
        opacity: 0.10,
    },
    SuitPlacement {
        atlas: 3,
        center: Vec2::new(22.0, 84.0),
        size: 11.0,
        tilt: 0.18,
        opacity: 0.09,
    },
];

#[derive(Component)]
pub(super) struct FloatingSuit {
    placement: SuitPlacement,
    phase: f32,
    period: f32,
}

impl FloatingSuit {
    pub fn new(placement: SuitPlacement, index: usize) -> Self {
        Self {
            placement,
            phase: index as f32 * 1.91,
            period: 36.0 + index as f32 * 4.7,
        }
    }

    pub fn sample(&self, seconds: f32) -> (UiTransform, Color) {
        let drift = seconds * consts::TAU / self.period + self.phase;
        let breath = seconds * consts::TAU / (12.0 + self.phase) + self.phase;
        let transform = UiTransform {
            // Transform offsets retain subpixel precision and do not dirty the layout.
            translation: Val2::new(
                Val::Vw(1.5 * drift.sin()),
                Val::Vh(2.0 * (drift * 0.79 + self.phase).cos()),
            ),
            rotation: Rot2::radians(self.placement.tilt + 0.055 * (drift * 0.63).sin()),
            scale: Vec2::splat(1.0 + 0.025 * breath.sin()),
        };
        let color = Color::srgb(0.69, 0.58, 0.95)
            .with_alpha(self.placement.opacity * (0.80 + 0.20 * breath.sin()));
        (transform, color)
    }
}

pub(super) fn animate_background(
    time: Res<Time>,
    assets: Res<BackgroundAssets>,
    mut materials: ResMut<Assets<BackgroundMaterial>>,
    mut suits: Query<(&FloatingSuit, &mut UiTransform, &mut ImageNode)>,
) {
    let seconds = time.elapsed_secs();
    if let Some(mut material) = materials.get_mut(&assets.material) {
        material.animation.x = seconds;
    }
    for (suit, mut transform, mut image) in &mut suits {
        let (next_transform, color) = suit.sample(seconds);
        *transform = next_transform;
        image.color = color;
    }
}
