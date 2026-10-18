use bevy::prelude::*;
use bevy::render::render_resource::AsBindGroup;
use bevy::shader::ShaderRef;

#[derive(AsBindGroup, Asset, TypePath, Debug, Clone)]
pub(in super::super) struct UnoDragGuideMaterial {
    #[uniform(0)]
    pub start_control: Vec4,
    #[uniform(0)]
    pub end_size: Vec4,
    #[uniform(0)]
    pub color: LinearRgba,
    #[uniform(0)]
    pub phase: Vec4,
}

impl UiMaterial for UnoDragGuideMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/uno_drag_guide.wesl".into()
    }
}
