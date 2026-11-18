use super::{EntryIcons, IconSprite};
use crate::app::presentation::OreUiSkin;
use bevy::image::{ImageLoaderSettings, ImageSampler};
use bevy::prelude::*;
use leocard_mc_guess::{McGuessEntry, McGuessEntryKind, McGuessGraphNodeView};
use std::ops::Deref;

#[derive(Resource)]
pub(crate) struct McGuessAssets {
    skin: OreUiSkin,
    icons: EntryIcons,
    node_symbols: Handle<Image>,
}

impl Deref for McGuessAssets {
    type Target = OreUiSkin;

    fn deref(&self) -> &Self::Target {
        &self.skin
    }
}

impl McGuessAssets {
    pub(in super::super) fn icon(&self, entry: &McGuessEntry) -> ImageNode {
        self.icons.image(entry)
    }

    pub(in super::super) fn node_sprite(
        &self,
        node: &McGuessGraphNodeView,
        layouts: &Assets<TextureAtlasLayout>,
    ) -> IconSprite {
        if let Some(entry) = &node.entry {
            return self.icons.sprite(entry, layouts);
        }
        let index = match node.kind {
            Some(McGuessEntryKind::Block) | None => 0,
            Some(McGuessEntryKind::Item) => 1,
            Some(McGuessEntryKind::Mob) => 2,
            Some(McGuessEntryKind::Enchantment) => 3,
            Some(McGuessEntryKind::Effect) => 4,
            Some(McGuessEntryKind::VillagerProfession) => 5,
            Some(McGuessEntryKind::Landscape) => 6,
            Some(McGuessEntryKind::Biome) => 7,
        };
        IconSprite {
            image: self.node_symbols.clone(),
            region: Vec4::new(index as f32 * 128.0, 0.0, 128.0, 128.0),
        }
    }
}

pub(in super::super) fn load_guess_assets(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut layouts: ResMut<Assets<TextureAtlasLayout>>,
) {
    let load = |path: &'static str| {
        asset_server
            .load_builder()
            .with_settings(|settings: &mut ImageLoaderSettings| {
                settings.sampler = ImageSampler::nearest();
            })
            .load(path)
    };
    commands.insert_resource(McGuessAssets {
        skin: OreUiSkin::load(&asset_server),
        icons: EntryIcons::new(&asset_server, &mut layouts),
        node_symbols: load("ui/mc_guess/graph/node-symbols.png"),
    });
}
