use super::MajorShowcaseSequence;
use bevy::prelude::*;
use std::f32::consts;

const SHAKE_DURATION: f32 = 0.22;

pub(super) struct MajorGlyphImpact {
    pub age: f32,
}

impl MajorGlyphImpact {
    pub fn at(local: f32, index: usize) -> Self {
        Self {
            age: local
                - MajorShowcaseSequence::FIRST_GLYPH
                - index as f32 * MajorShowcaseSequence::GLYPH_INTERVAL,
        }
    }

    pub fn transform(&self, size: Vec2, aspect: f32, bounds: Vec4) -> UiTransform {
        let fall = (self.age / MajorShowcaseSequence::GLYPH_FALL).clamp(0.0, 1.0);
        let settle = ((self.age - MajorShowcaseSequence::GLYPH_FALL)
            / MajorShowcaseSequence::GLYPH_SETTLE)
            .clamp(0.0, 1.0);
        let contact = (settle * consts::PI).sin() * (1.0 - settle);
        let scale = 1.0 + 1.4 * (1.0 - fall.powi(3)) - 0.10 * contact;
        let extent = Vec2::new(size.x.min(size.y * aspect), size.y.min(size.x / aspect));
        let center = (bounds.xy() + bounds.zw() * 0.5 - Vec2::splat(0.5)) * extent;
        // Keep the glyph's own center fixed while its full-title painting surface scales.
        // Scaling the node also expands its painting bounds, so enlarged brush strokes are not clipped.
        let offset =
            center * (1.0 - scale) + Vec2::new(0.0, -64.0 * (1.0 - fall.powi(2)) + 4.0 * contact);
        UiTransform {
            translation: Val2::px(offset.x, offset.y),
            scale: Vec2::splat(scale),
            ..default()
        }
    }

    pub fn shake(&self) -> Vec3 {
        let elapsed = self.age - MajorShowcaseSequence::GLYPH_FALL;
        if !(0.0..SHAKE_DURATION).contains(&elapsed) {
            return Vec3::ZERO;
        }
        let strength = (1.0 - elapsed / SHAKE_DURATION).powi(2);
        Vec3::new(
            (elapsed * 105.0).sin() * 6.0,
            (elapsed * 137.0).cos() * 4.0,
            (elapsed * 89.0).sin() * 0.002,
        ) * strength
    }
}
