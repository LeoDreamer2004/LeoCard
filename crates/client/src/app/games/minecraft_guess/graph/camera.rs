use bevy::prelude::*;

pub(in super::super) struct GraphCamera {
    pub pan: Vec2,
    pub zoom: f32,
}

impl Default for GraphCamera {
    fn default() -> Self {
        Self {
            pan: Vec2::ZERO,
            zoom: 1.5,
        }
    }
}

impl GraphCamera {
    pub fn project(&self, position: Vec2, size: Vec2) -> Vec2 {
        (position - Vec2::splat(0.5)) * size * self.zoom + size * 0.5 + self.pan
    }

    pub fn unproject(&self, point: Vec2, size: Vec2) -> Vec2 {
        (point - size * 0.5 - self.pan) / (size * self.zoom) + Vec2::splat(0.5)
    }

    pub fn zoom_at(&mut self, point: Vec2, size: Vec2, amount: f32) {
        let anchor = self.unproject(point, size);
        self.zoom = (self.zoom * amount.exp()).clamp(0.45, 2.6);
        self.pan = point - size * 0.5 - (anchor - Vec2::splat(0.5)) * size * self.zoom;
    }
}
