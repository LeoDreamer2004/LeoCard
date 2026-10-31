use super::material::GuessBackgroundMaterial;
use bevy::image::{ImageLoaderSettings, ImageSampler};
use bevy::prelude::*;
use std::collections::HashMap;

const VERSIONS: &str =
    include_str!("../../../../../../../../assets/ui/minecraft_guess/background/versions.tsv");
const BLOCK_FRONTS: &str =
    include_str!("../../../../../../../../assets/ui/minecraft_guess/background/block-fronts.tsv");

#[derive(Resource)]
pub(super) struct BackgroundAssets {
    pub material: Handle<GuessBackgroundMaterial>,
    pub tile_count: usize,
    versions: HashMap<&'static str, &'static str>,
}

impl FromWorld for BackgroundAssets {
    fn from_world(world: &mut World) -> Self {
        let server = world.resource::<AssetServer>();
        let tiles = load_tiles(server, "ui/minecraft_guess/background/block-fronts.png");
        let dirt = load_tiles(server, "ui/minecraft_guess/background/dirt.png");
        let material =
            world
                .resource_mut::<Assets<GuessBackgroundMaterial>>()
                .add(GuessBackgroundMaterial {
                    params: Vec4::new(0.0, 0.0, 1.0, 0.0),
                    random_background: 1,
                    panorama: None,
                    tiles,
                    dirt,
                });
        let versions = VERSIONS
            .lines()
            .skip(1)
            .map(|line| line.split_once('\t').expect("bundled panorama version row"))
            .collect();
        let tile_count = BLOCK_FRONTS.lines().skip(1).count();
        assert!(
            tile_count > 0,
            "bundled full block front atlas is not empty"
        );
        Self {
            material,
            tile_count,
            versions,
        }
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

fn load_tiles(server: &AssetServer, path: &'static str) -> Handle<Image> {
    server
        .load_builder()
        .with_settings(|settings: &mut ImageLoaderSettings| {
            settings.sampler = ImageSampler::nearest();
            settings.is_srgb = false;
        })
        .load(path)
}
