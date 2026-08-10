//! GPU table-background treatment.

use super::*;

#[derive(AsBindGroup, Asset, TypePath, Debug, Clone)]
pub(super) struct TableBackgroundMaterial {
    /// x: vignette strength, y: brightness, z: tiled flag, w: unused.
    #[uniform(0)]
    pub(super) params: Vec4,
    #[texture(1)]
    #[sampler(2)]
    pub(super) texture: Handle<Image>,
}

impl UiMaterial for TableBackgroundMaterial {
    fn fragment_shader() -> ShaderRef {
        TABLE_BACKGROUND_SHADER.into()
    }
}

pub(super) fn table_material_params(brightness: f32, vignette: f32, tiled: bool) -> Vec4 {
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
