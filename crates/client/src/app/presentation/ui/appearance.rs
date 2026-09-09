//! GPU table-background treatment.

use super::{DEFAULT_TABLE_VIGNETTE, MAX_TABLE_VIGNETTE, MIN_TABLE_VIGNETTE};
use crate::app::runtime::{normalize_range, normalize_table_brightness};
use bevy::prelude::*;
use bevy::render::render_resource::AsBindGroup;
use bevy::shader::ShaderRef;

pub(crate) const TABLE_BACKGROUND_SHADER: &str = "shaders/table_background.wgsl";

#[derive(AsBindGroup, Asset, TypePath, Debug, Clone)]
pub(crate) struct TableBackgroundMaterial {
    /// x: vignette strength, y: brightness, z: tiled flag, w: unused.
    #[uniform(0)]
    pub params: Vec4,
    #[texture(1)]
    #[sampler(2)]
    pub texture: Handle<Image>,
}

impl UiMaterial for TableBackgroundMaterial {
    fn fragment_shader() -> ShaderRef {
        TABLE_BACKGROUND_SHADER.into()
    }
}

pub(crate) fn table_material_params(brightness: f32, vignette: f32, tiled: bool) -> Vec4 {
    Vec4::new(
        normalize_range(
            vignette,
            MIN_TABLE_VIGNETTE,
            MAX_TABLE_VIGNETTE,
            DEFAULT_TABLE_VIGNETTE,
        ),
        normalize_table_brightness(brightness),
        if tiled { 1.0 } else { 0.0 },
        0.0,
    )
}
