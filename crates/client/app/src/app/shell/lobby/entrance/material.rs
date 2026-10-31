use bevy::prelude::*;
use bevy::render::render_resource::AsBindGroup;
use bevy::shader::ShaderRef;

#[derive(Asset, AsBindGroup, TypePath, Debug, Clone)]
pub(super) struct EntranceVeil {
    #[uniform(0)]
    pub spotlight: Vec4,
}

impl UiMaterial for EntranceVeil {
    fn fragment_shader() -> ShaderRef {
        "shaders/lobby_entrance.wesl".into()
    }
}
