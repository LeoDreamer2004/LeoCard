//! GPU table-background treatment.

use crate::app::runtime::{TableSurfaceAssets, normalize_table_brightness};
use bevy::prelude::*;
use bevy::render::render_resource::AsBindGroup;
use bevy::shader::ShaderRef;

pub(crate) const TABLE_BACKGROUND_SHADER: &str = "shaders/table_background.wesl";

pub(crate) use crate::app::runtime::TableSurfaceKind;

#[derive(AsBindGroup, Asset, TypePath, Debug, Clone, PartialEq)]
pub(crate) struct TableBackgroundMaterial {
    /// x: brightness; y: 0 centered ellipse, 1 measured rectangular mahjong table.
    #[uniform(0)]
    pub params: Vec4,
    #[uniform(0)]
    felt_bounds: Vec4,
    #[uniform(0)]
    frame_bounds: Vec4,
    #[texture(1)]
    #[sampler(2)]
    pub texture: Handle<Image>,
    #[texture(3)]
    #[sampler(4)]
    rail: Handle<Image>,
}

impl UiMaterial for TableBackgroundMaterial {
    fn fragment_shader() -> ShaderRef {
        TABLE_BACKGROUND_SHADER.into()
    }
}

impl TableBackgroundMaterial {
    pub(crate) fn new(
        kind: TableSurfaceKind,
        brightness: f32,
        assets: &TableSurfaceAssets,
        custom_felt: Option<&Handle<Image>>,
    ) -> Self {
        Self {
            params: Vec4::new(
                normalize_table_brightness(brightness),
                match kind {
                    TableSurfaceKind::Cards => 0.0,
                    TableSurfaceKind::Mahjong => 1.0,
                },
                0.0,
                0.0,
            ),
            felt_bounds: assets.felt_bounds,
            frame_bounds: assets.frame_bounds,
            texture: custom_felt.unwrap_or(&assets.felt).clone(),
            rail: assets.rail.clone(),
        }
    }
}
