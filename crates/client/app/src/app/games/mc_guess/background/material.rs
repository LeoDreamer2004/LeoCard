use bevy::prelude::*;
use bevy::render::render_resource::AsBindGroup;
use bevy::shader::ShaderRef;

#[derive(AsBindGroup, Asset, TypePath, Debug, Clone)]
pub(super) struct GuessBackgroundMaterial {
    /// x: panorama spin, y: transition, z: inverse UI scale, w: block front atlas index.
    #[uniform(0)]
    pub params: Vec4,
    #[uniform(0)]
    pub random_background: u32,
    #[texture(1, dimension = "cube")]
    #[sampler(2)]
    pub panorama: Option<Handle<Image>>,
    #[texture(3)]
    #[sampler(4)]
    pub tiles: Handle<Image>,
    #[texture(5)]
    #[sampler(6)]
    pub dirt: Handle<Image>,
}

impl UiMaterial for GuessBackgroundMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/minecraft_background.wesl".into()
    }
}
