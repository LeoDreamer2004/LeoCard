use bevy::image::{ImageLoaderSettings, ImageSampler};
use bevy::prelude::*;

pub(crate) struct MahjongWinShowcaseAssets {
    pub(super) ink: Handle<Image>,
    pub(super) progress: Handle<Image>,
    pub(super) title: Handle<Image>,
}

impl MahjongWinShowcaseAssets {
    pub(crate) fn load(server: &AssetServer) -> Self {
        let load = |name, is_srgb| {
            server
                .load_builder()
                .with_settings(move |settings: &mut ImageLoaderSettings| {
                    settings.is_srgb = is_srgb;
                    settings.sampler = ImageSampler::linear();
                })
                .load(format!("ui/mahjong/win-showcase/{name}.png"))
        };
        Self {
            ink: load("ink", true),
            progress: load("stroke-progress", false),
            title: load("title", true),
        }
    }
}
