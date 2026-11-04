use bevy::prelude::*;
use bevy::render::render_resource::AsBindGroup;
use bevy::shader::ShaderRef;

pub(super) const MAX_RIPPLES: usize = 32;

#[derive(AsBindGroup, Asset, TypePath, Debug, Clone)]
pub(super) struct LiquidMaterial {
    #[uniform(0)]
    pub tint: Vec4,
    /// Time, pointer attraction, active ripple count, subtree opacity.
    #[uniform(0)]
    pub animation: Vec4,
    #[uniform(0)]
    pub pointer: Vec2,
    /// Sweep mode, per-game phase, press/hover intensity, sweep time.
    #[uniform(0)]
    pub params: Vec4,
    #[uniform(0)]
    pub uv_rect: Vec4,
    /// Click UV and age in seconds for each active ripple.
    #[uniform(0)]
    pub ripples: [Vec4; MAX_RIPPLES],
    #[texture(1)]
    #[sampler(2)]
    pub texture: Handle<Image>,
}

impl LiquidMaterial {
    pub(super) fn set_ripples(&mut self, ripples: &[Vec4]) {
        self.animation.z = ripples.len() as f32;
        self.ripples[..ripples.len()].copy_from_slice(ripples);
    }
}

impl UiMaterial for LiquidMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/home_liquid.wesl".into()
    }
}
