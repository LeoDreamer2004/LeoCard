use super::MahjongWinShowcaseAssets;
use bevy::prelude::*;
use bevy::render::render_resource::AsBindGroup;
use bevy::shader::ShaderRef;

#[derive(Asset, TypePath, AsBindGroup, Debug, Clone, PartialEq)]
pub(crate) struct MahjongWinInkMaterial {
    /// x: paint progress, y: erasure progress, z: opacity, w: local time.
    #[uniform(0)]
    pub(super) animation: Vec4,
    #[texture(1)]
    #[sampler(2)]
    ink: Handle<Image>,
    #[texture(3)]
    #[sampler(4)]
    progress: Handle<Image>,
}

impl MahjongWinInkMaterial {
    pub(super) fn new(assets: &MahjongWinShowcaseAssets) -> Self {
        Self {
            animation: Vec4::new(-0.1, -0.1, 0.0, 0.0),
            ink: assets.ink.clone(),
            progress: assets.progress.clone(),
        }
    }
}

impl UiMaterial for MahjongWinInkMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/mahjong_win_ink.wesl".into()
    }
}
