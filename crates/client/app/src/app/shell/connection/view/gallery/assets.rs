use super::{ENTRY_DESIGNS, EntryArt};
use bevy::prelude::*;

#[derive(Resource)]
pub(super) struct GalleryAssets {
    images: [Option<Handle<Image>>; ENTRY_DESIGNS.len()],
}

impl FromWorld for GalleryAssets {
    fn from_world(world: &mut World) -> Self {
        let server = world.resource::<AssetServer>();
        Self {
            images: ENTRY_DESIGNS
                .each_ref()
                .map(|entry| entry.image.map(|image| server.load(image))),
        }
    }
}

pub(super) fn load_entry_art(
    assets: Res<GalleryAssets>,
    mut entries: Query<(&EntryArt, &mut ImageNode), Changed<EntryArt>>,
) {
    for (art, mut image) in &mut entries {
        if let Some(handle) = &assets.images[art.0] {
            image.image = handle.clone();
        }
    }
}
