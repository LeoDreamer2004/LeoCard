use bevy::prelude::*;
use std::f32::consts;

pub(super) struct DirectionLoopGeometry {
    pub size: Vec2,
    pub center: Vec2,
    pub radii: Vec2,
    pub width: f32,
    pub perimeter: f32,
}

impl DirectionLoopGeometry {
    pub fn new(size: Vec2, width: f32) -> Self {
        let padding = Vec2::splat(width * 0.9 + 4.0);
        let radii =
            (size * Vec2::new(0.44, 0.43)).min((size * 0.5 - padding).max(Vec2::splat(width)));
        let difference = ((radii.x - radii.y) / (radii.x + radii.y)).powi(2);
        let perimeter = consts::PI
            * (radii.x + radii.y)
            * (1.0 + 3.0 * difference / (10.0 + (4.0 - 3.0 * difference).sqrt()));
        Self {
            size,
            center: size * 0.5,
            radii,
            width,
            perimeter,
        }
    }

    pub fn matches(&self, size: Vec2) -> bool {
        self.size.distance_squared(size) < 0.25
    }

    pub fn advance(&self, phase: f32, speed: f32, delta: f32) -> f32 {
        let angle = phase * consts::TAU;
        let tangent = Vec2::new(-self.radii.x * angle.sin(), self.radii.y * angle.cos()).length();
        (phase + speed * delta * self.perimeter / (40.0 * consts::TAU * tangent)).rem_euclid(1.0)
    }
}
