//! One full-screen material for the base, light ribbons and vignette.

use bevy::prelude::*;
use bevy::render::render_resource::AsBindGroup;
use bevy::shader::ShaderRef;

#[derive(AsBindGroup, Asset, TypePath, Debug, Default, Clone)]
pub(super) struct BackgroundMaterial {
    /// x: continuous animation time in seconds; remaining components are padding.
    #[uniform(0)]
    pub animation: Vec4,
}

impl UiMaterial for BackgroundMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/page_background.wgsl".into()
    }
}
