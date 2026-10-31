use bevy::prelude::*;

#[derive(Clone, Copy)]
pub(in super::super) struct Segment {
    start: Vec2,
    end: Vec2,
}

impl Segment {
    pub fn new(start: Vec2, end: Vec2) -> Self {
        Self { start, end }
    }

    pub fn closest(self, point: Vec2) -> (Vec2, f32) {
        let direction = self.end - self.start;
        let t = ((point - self.start).dot(direction) / direction.length_squared().max(0.001))
            .clamp(0.0, 1.0);
        (self.start + direction * t, t)
    }

    pub fn crosses(self, other: Self) -> bool {
        let direction = self.end - self.start;
        let across = other.end - other.start;
        direction.perp_dot(other.start - self.start) * direction.perp_dot(other.end - self.start)
            < -0.01
            && across.perp_dot(self.start - other.start) * across.perp_dot(self.end - other.start)
                < -0.01
    }
}
