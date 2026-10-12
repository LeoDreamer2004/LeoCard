use bevy::prelude::*;
use bevy::render::render_resource::AsBindGroup;
use bevy::shader::ShaderRef;

#[derive(AsBindGroup, Asset, TypePath, Debug, Clone)]
pub(super) struct GuessBackgroundMaterial {
    /// x: panorama spin in radians, y: transition progress, z: inverse UI scale.
    #[uniform(0)]
    pub params: Vec4,
    #[texture(1, dimension = "cube")]
    #[sampler(2)]
    pub panorama: Option<Handle<Image>>,
    #[texture(3)]
    #[sampler(4)]
    pub dirt: Handle<Image>,
}

impl UiMaterial for GuessBackgroundMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/minecraft_background.wesl".into()
    }
}
