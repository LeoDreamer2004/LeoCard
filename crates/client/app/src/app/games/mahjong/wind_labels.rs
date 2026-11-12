//! Carved wind glyphs use monochrome coverage, independent of the UI text font.

use crate::app::presentation::TEXT;
use bevy::image::{ImageLoaderSettings, ImageSampler};
use bevy::prelude::*;
use bevy::render::render_resource::AsBindGroup;
use bevy::shader::ShaderRef;
use leocard_mahjong::MahjongWind;

#[derive(Asset, TypePath, AsBindGroup, Debug, Clone, PartialEq)]
pub(super) struct MahjongWindLabelMaterial {
    #[uniform(0)]
    color: LinearRgba,
    /// Visible ink bounds in normalized texture coordinates (x, y, width, height).
    #[uniform(0)]
    ink_bounds: Vec4,
    #[texture(1)]
    #[sampler(2)]
    texture: Handle<Image>,
}

impl UiMaterial for MahjongWindLabelMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/mahjong_wind_label.wesl".into()
    }
}

pub(super) struct MahjongWindLabels([Handle<MahjongWindLabelMaterial>; 4]);

impl MahjongWindLabels {
    pub(super) fn load(
        server: &AssetServer,
        materials: &mut Assets<MahjongWindLabelMaterial>,
    ) -> Self {
        Self(
            [
                ("east", Vec4::new(103.0, 68.0, 1049.0, 1134.0)),
                ("south", Vec4::new(161.0, 81.0, 935.0, 1093.0)),
                ("west", Vec4::new(149.0, 182.0, 955.0, 936.0)),
                ("north", Vec4::new(114.0, 163.0, 1025.0, 934.0)),
            ]
            .map(|(name, ink_bounds)| {
                let texture = server
                    .load_builder()
                    .with_settings(|settings: &mut ImageLoaderSettings| {
                        // These pixels encode coverage, not a display color.
                        settings.is_srgb = false;
                        settings.sampler = ImageSampler::linear();
                    })
                    .load(format!("ui/mahjong/wind-label-{name}.png"));
                materials.add(MahjongWindLabelMaterial {
                    color: if name == "east" {
                        Color::srgb(1.0, 0.78, 0.36)
                    } else {
                        TEXT
                    }
                    .to_linear(),
                    // Normalize measurements from the original 1254px masks.
                    ink_bounds: ink_bounds / 1254.0,
                    texture,
                })
            }),
        )
    }

    pub(super) fn get(&self, wind: MahjongWind) -> &Handle<MahjongWindLabelMaterial> {
        &self.0[match wind {
            MahjongWind::East => 0,
            MahjongWind::South => 1,
            MahjongWind::West => 2,
            MahjongWind::North => 3,
        }]
    }
}
