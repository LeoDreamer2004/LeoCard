use bevy::prelude::*;

pub(in super::super) struct GraphCamera {
    pub pan: Vec2,
    pub zoom: f32,
    pan_velocity: Vec2,
    zoom_target: f32,
    zoom_velocity: f32,
    zoom_anchor: Vec2,
}

impl Default for GraphCamera {
    fn default() -> Self {
        Self {
            pan: Vec2::ZERO,
            zoom: 1.0,
            pan_velocity: Vec2::ZERO,
            zoom_target: 0.0,
            zoom_velocity: 0.0,
            zoom_anchor: Vec2::ZERO,
        }
    }
}

impl GraphCamera {
    const MIN_ZOOM: f32 = 0.45;
    const MAX_ZOOM: f32 = 2.6;

    pub fn project(&self, position: Vec2, size: Vec2) -> Vec2 {
        position * self.zoom + size * 0.5 + self.pan
    }

    pub fn unproject(&self, point: Vec2, size: Vec2) -> Vec2 {
        (point - size * 0.5 - self.pan) / self.zoom
    }

    pub fn zoom_at(&mut self, point: Vec2, size: Vec2, amount: f32) {
        self.zoom_anchor = self.unproject(point, size);
        self.zoom_target =
            (self.zoom_target + amount).clamp(Self::MIN_ZOOM.ln(), Self::MAX_ZOOM.ln());
        self.pan_velocity = Vec2::ZERO;
    }

    pub fn stop(&mut self) {
        self.pan_velocity = Vec2::ZERO;
        self.zoom_velocity = 0.0;
        self.zoom_target = self.zoom.ln();
    }

    pub fn sample_pan(&mut self, displacement: Vec2, delta: f32) {
        if delta <= 0.0 {
            return;
        }
        let velocity = (displacement / delta).clamp_length_max(2400.0);
        self.pan_velocity = self
            .pan_velocity
            .lerp(velocity, 1.0 - (-30.0 * delta).exp());
    }

    pub fn advance(&mut self, delta: f32, dragging: bool) {
        let delta = delta.min(0.05);
        if delta <= 0.0 {
            return;
        }
        // An idle hold removes stale fling speed before the pointer is released.
        let friction = if dragging { 12.0 } else { 7.5 };
        let decay = (-friction * delta).exp();
        if !dragging {
            self.pan += self.pan_velocity * ((1.0 - decay) / friction);
        }
        self.pan_velocity *= decay;
        if self.pan_velocity.length_squared() < 1.0 {
            self.pan_velocity = Vec2::ZERO;
        }

        // Critically damped motion in log space gives proportional zoom steps,
        // keeping speed continuous when new wheel input arrives mid-animation.
        let current = self.zoom.ln();
        let error = current - self.zoom_target;
        if error.abs() < 0.00001 && self.zoom_velocity.abs() < 0.0001 {
            self.set_zoom(self.zoom_target.exp());
            self.zoom_velocity = 0.0;
            return;
        }
        let spring = 18.0;
        let impulse = self.zoom_velocity + spring * error;
        let decay = (-spring * delta).exp();
        let next = self.zoom_target + (error + impulse * delta) * decay;
        self.zoom_velocity = (self.zoom_velocity - spring * impulse * delta) * decay;
        self.set_zoom(next.exp());
    }

    fn set_zoom(&mut self, zoom: f32) {
        let zoom = zoom.clamp(Self::MIN_ZOOM, Self::MAX_ZOOM);
        self.pan += self.zoom_anchor * (self.zoom - zoom);
        self.zoom = zoom;
    }
}
