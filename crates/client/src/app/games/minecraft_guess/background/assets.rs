use super::material::GuessBackgroundMaterial;
use bevy::image::{ImageLoaderSettings, ImageSampler};
use bevy::prelude::*;
use std::collections::HashMap;

const VERSIONS: &str =
    include_str!("../../../../../../../assets/ui/minecraft_guess/background/versions.tsv");

#[derive(Resource)]
pub(super) struct BackgroundAssets {
    pub material: Handle<GuessBackgroundMaterial>,
    versions: HashMap<&'static str, &'static str>,
}

impl FromWorld for BackgroundAssets {
    fn from_world(world: &mut World) -> Self {
        let dirt = world
            .resource::<AssetServer>()
            .load_builder()
            .with_settings(|settings: &mut ImageLoaderSettings| {
                settings.sampler = ImageSampler::nearest();
                settings.is_srgb = false;
            })
            .load("ui/minecraft_guess/background/dirt.png");
        let material =
            world
                .resource_mut::<Assets<GuessBackgroundMaterial>>()
                .add(GuessBackgroundMaterial {
                    params: Vec4::new(0.0, 0.0, 1.0, 0.0),
                    panorama: None,
                    dirt,
                });
        let versions = VERSIONS
            .lines()
            .skip(1)
            .map(|line| line.split_once('\t').expect("bundled panorama version row"))
            .collect();
        Self { material, versions }
    }
}

impl BackgroundAssets {
    pub fn panorama_path(&self, version: &str) -> &'static str {
        self.versions
            .get(version)
            .copied()
            .expect("bundled catalogue version has a panorama")
    }
}
