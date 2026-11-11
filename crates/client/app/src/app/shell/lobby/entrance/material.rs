use bevy::prelude::*;
use bevy::render::render_resource::AsBindGroup;
use bevy::shader::ShaderRef;

#[derive(Asset, AsBindGroup, TypePath, Debug, Clone, Default)]
pub(super) struct EntranceVeil {
    #[uniform(0)]
    pub screen_to_avatar: Vec4,
    #[uniform(0)]
    pub center: Vec2,
    #[uniform(0)]
    pub half_size: Vec2,
    #[uniform(0)]
    pub corner_radius: f32,
    #[uniform(0)]
    pub opacity: f32,
}

impl UiMaterial for EntranceVeil {
    fn fragment_shader() -> ShaderRef {
        "shaders/lobby_entrance.wesl".into()
    }
}
