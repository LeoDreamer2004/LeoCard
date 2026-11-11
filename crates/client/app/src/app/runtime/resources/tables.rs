//! Two table frames and parallel default felt textures, independent of game rules.

use bevy::image::{ImageLoaderSettings, ImageSampler};
use bevy::prelude::*;

#[derive(Default)]
pub(crate) struct TableAssets {
    pub oval: TableFrameAssets,
    pub mahjong: TableFrameAssets,
    pub felts: TableFeltAssets,
}

impl TableAssets {
    pub(crate) fn load(server: &AssetServer) -> Self {
        let load = |variant, layer| {
            server
                .load_builder()
                .with_settings(|settings: &mut ImageLoaderSettings| {
                    settings.sampler = ImageSampler::linear();
                })
                .load(format!("ui/tables/{variant}/table-{layer}.png"))
        };
        Self {
            oval: TableFrameAssets {
                rail: load("casino", "rail"),
                felt_bounds: Vec4::new(108.0, 102.0, 1647.0, 791.0)
                    / Vec4::new(1774.0, 887.0, 1774.0, 887.0),
                frame_bounds: Vec4::new(49.0, 39.0, 1707.0, 858.0)
                    / Vec4::new(1774.0, 887.0, 1774.0, 887.0),
            },
            mahjong: TableFrameAssets {
                rail: load("mahjong", "rail"),
                felt_bounds: Vec4::new(83.0, 83.0, 1172.0, 1164.0) / 1254.0,
                frame_bounds: Vec4::new(10.0, 8.0, 1244.0, 1245.0) / 1254.0,
            },
            felts: TableFeltAssets {
                casino: load("casino", "felt"),
                color_card: load("color-card", "felt"),
                paper_card: load("paper-card", "felt"),
                mahjong: load("mahjong", "felt"),
            },
        }
    }
}

#[derive(Default)]
pub(crate) struct TableFrameAssets {
    pub rail: Handle<Image>,
    /// Inner opening and clean outer silhouette in normalized source coordinates.
    pub felt_bounds: Vec4,
    pub frame_bounds: Vec4,
}

#[derive(Default)]
pub(crate) struct TableFeltAssets {
    pub casino: Handle<Image>,
    pub color_card: Handle<Image>,
    pub paper_card: Handle<Image>,
    pub mahjong: Handle<Image>,
}
