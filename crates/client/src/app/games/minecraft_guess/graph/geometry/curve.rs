use super::super::shape::NodeShape;
use bevy::prelude::*;

pub(in super::super) struct CurveGeometry {
    pub start: Vec2,
    pub control: Vec2,
    pub end: Vec2,
}

impl CurveGeometry {
    pub fn between(
        from: Vec2,
        to: Vec2,
        offset: f32,
        source: NodeShape,
        target: NodeShape,
        node_scale: f32,
    ) -> Self {
        let direction = (to - from).normalize_or_zero();
        let control = (from + to) * 0.5 + direction.perp() * offset;
        let start_direction = (control - from).normalize_or_zero();
        let end_direction = (control - to).normalize_or_zero();
        Self {
            start: from + start_direction * (source.radius() * node_scale + 4.0),
            control,
            end: to + end_direction * (target.radius() * node_scale + 4.0),
        }
    }

    pub fn label_position(&self) -> Vec2 {
        (self.start + self.control * 2.0 + self.end) * 0.25 - Vec2::Y * 12.0
    }

    pub fn bounds(&self) -> Rect {
        Rect::from_corners(
            self.start.min(self.control).min(self.end) - Vec2::splat(8.0),
            self.start.max(self.control).max(self.end) + Vec2::splat(8.0),
        )
    }
}
