use bevy::prelude::*;

/// Shape shared by table projection and front-facing tile layouts.
pub(crate) struct MahjongTileGeometry;

impl MahjongTileGeometry {
    // Source glyphs are 600 × 800; the raised rim is additional depth.
    pub const FACE_ASPECT_RATIO: f32 = 3.0 / 4.0;
    pub const THICKNESS: f32 = 0.6;

    pub fn flat_half_size(scale: f32) -> Vec3 {
        Vec3::new(0.5, Self::THICKNESS * 0.5, 0.5 / Self::FACE_ASPECT_RATIO) * scale
    }
}
