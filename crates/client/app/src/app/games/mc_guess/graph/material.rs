use bevy::prelude::*;
use bevy::render::render_resource::AsBindGroup;
use bevy::shader::ShaderRef;

#[derive(AsBindGroup, Asset, TypePath, Debug, Clone)]
pub(in super::super) struct GraphCurveMaterial {
    #[uniform(0)]
    pub start_control: Vec4,
    #[uniform(0)]
    pub end_size: Vec4,
    #[uniform(0)]
    pub color: LinearRgba,
    #[uniform(0)]
    pub appearance: Vec4,
}

impl UiMaterial for GraphCurveMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/minecraft_graph_curve.wesl".into()
    }
}

#[derive(AsBindGroup, Asset, TypePath, Debug, Clone)]
pub(in super::super) struct GraphNodeMaterial {
    #[uniform(0)]
    pub color: LinearRgba,
    #[uniform(0)]
    pub state: Vec4,
    #[uniform(0)]
    pub sprite: Vec4,
    #[uniform(0)]
    /// Question-wave strength/time, then combo-halo strength/time.
    pub activity: Vec4,
    #[texture(1)]
    #[sampler(2)]
    pub icon: Handle<Image>,
}

impl UiMaterial for GraphNodeMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/minecraft_graph_node.wesl".into()
    }
}
