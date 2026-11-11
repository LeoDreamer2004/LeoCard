use bevy::prelude::*;
use bevy::render::render_resource::AsBindGroup;
use bevy::shader::ShaderRef;

#[derive(AsBindGroup, Asset, TypePath, Debug, Clone)]
pub(crate) struct UnoDirectionMaterial {
    #[uniform(0)]
    pub canvas: Vec4,
    #[uniform(0)]
    pub shape: Vec4,
    #[uniform(0)]
    pub motion: Vec4,
    #[uniform(0)]
    pub color: LinearRgba,
}

impl UiMaterial for UnoDirectionMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/uno_direction.wesl".into()
    }
}
