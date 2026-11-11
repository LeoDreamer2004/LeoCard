//! GPU table-background treatment.

use crate::app::runtime::{TableAssets, normalize_table_brightness};
use bevy::prelude::*;
use bevy::render::render_resource::AsBindGroup;
use bevy::shader::ShaderRef;

pub(crate) const TABLE_BACKGROUND_SHADER: &str = "shaders/table_background.wesl";

pub(crate) use crate::app::runtime::TableSurfaceKind;

#[derive(AsBindGroup, Asset, TypePath, Debug, Clone, PartialEq)]
pub(crate) struct TableBackgroundMaterial {
    /// x: brightness; y: 0 oval card table, 1 projected mahjong table.
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
        tables: &TableAssets,
        default_felt: &Handle<Image>,
        custom_felt: Option<&Handle<Image>>,
    ) -> Self {
        let (shape, frame) = match kind {
            TableSurfaceKind::Cards => (0.0, &tables.oval),
            TableSurfaceKind::Mahjong => (1.0, &tables.mahjong),
        };
        Self {
            params: Vec4::new(normalize_table_brightness(brightness), shape, 0.0, 0.0),
            felt_bounds: frame.felt_bounds,
            frame_bounds: frame.frame_bounds,
            texture: custom_felt.unwrap_or(default_felt).clone(),
            rail: frame.rail.clone(),
        }
    }
}
