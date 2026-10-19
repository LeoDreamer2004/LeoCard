use bevy::prelude::*;

pub(super) struct CurveGeometry {
    pub start: Vec2,
    pub control: Vec2,
    pub end: Vec2,
}

impl CurveGeometry {
    pub fn between(from: Vec2, to: Vec2, bend: f32) -> Self {
        let direction = (to - from).normalize_or_zero();
        let offset = ((to - from).length() * 0.09).clamp(14.0, 30.0);
        Self {
            start: from + direction * 28.0,
            control: (from + to) * 0.5 + direction.perp() * offset * bend,
            end: to - direction * 28.0,
        }
    }

    pub fn point(&self, t: f32) -> Vec2 {
        self.start * (1.0 - t).powi(2) + self.control * 2.0 * (1.0 - t) * t + self.end * t * t
    }

    pub fn label_position(&self, nodes: &[Vec2], labels: &[Vec2]) -> Vec2 {
        [0.5, 0.35, 0.65, 0.25, 0.75]
            .into_iter()
            .map(|t| {
                let point = self.point(t) + Vec2::new(0.0, -9.0);
                let node_clearance = nodes
                    .iter()
                    .map(|p| ((point - *p) / Vec2::new(1.5, 1.0)).length())
                    .fold(f32::INFINITY, f32::min);
                let label_clearance = labels
                    .iter()
                    .map(|p| point.distance(*p))
                    .fold(120.0, f32::min);
                (
                    point,
                    node_clearance.min(65.0) + label_clearance.min(65.0) * 0.6
                        - (t - 0.5_f32).abs() * 20.0,
                )
            })
            .max_by(|a, b| a.1.total_cmp(&b.1))
            .expect("candidate labels")
            .0
    }

    pub fn bounds(&self) -> Rect {
        Rect::from_corners(
            self.start.min(self.control).min(self.end) - Vec2::splat(8.0),
            self.start.max(self.control).max(self.end) + Vec2::splat(8.0),
        )
    }
}
