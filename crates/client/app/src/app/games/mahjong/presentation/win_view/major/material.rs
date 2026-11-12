use super::assets::MajorFanArt;
use bevy::prelude::*;
use bevy::render::render_resource::AsBindGroup;
use bevy::shader::ShaderRef;

#[derive(Asset, TypePath, AsBindGroup, Debug, Clone, PartialEq)]
pub(crate) struct MahjongWinArtMaterial {
    /// Opacity, paint coverage, camera scale, layer kind (background or glyph).
    #[uniform(0)]
    pub(super) animation: Vec4,
    /// Glyph bounds in the complete calligraphy texture.
    #[uniform(0)]
    bounds: Vec4,
    /// Glyph-local translation, scale and whole-title polishing-light position.
    #[uniform(0)]
    pub(super) pose: Vec4,
    /// Texture aspect ratio and polishing-light strength.
    #[uniform(0)]
    pub(super) surface: Vec4,
    #[texture(1)]
    #[sampler(2)]
    texture: Handle<Image>,
}

impl MahjongWinArtMaterial {
    pub(super) fn background(art: &MajorFanArt) -> Self {
        Self {
            animation: Vec4::new(0.0, 0.0, 1.035, 0.0),
            bounds: Vec4::new(0.0, 0.0, 1.0, 1.0),
            pose: Vec4::new(0.0, 0.0, 1.0, -0.35),
            surface: Vec4::new(16.0 / 9.0, 0.0, 0.0, 0.0),
            texture: art.background.clone(),
        }
    }

    pub(super) fn glyph(art: &MajorFanArt, bounds: Vec4) -> Self {
        Self {
            animation: Vec4::new(0.0, 0.0, 1.0, 1.0),
            bounds,
            pose: Vec4::new(0.0, 0.0, 1.0, -0.35),
            surface: Vec4::new(art.spec.title_aspect, 0.0, 0.0, 0.0),
            texture: art.title.clone(),
        }
    }
}

impl UiMaterial for MahjongWinArtMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/mahjong_win_art.wesl".into()
    }
}
