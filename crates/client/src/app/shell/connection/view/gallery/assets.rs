use super::{ENTRY_DESIGNS, EntryArt};
use bevy::prelude::*;

#[derive(Resource)]
pub(super) struct GalleryAssets {
    images: [Handle<Image>; 5],
}

impl FromWorld for GalleryAssets {
    fn from_world(world: &mut World) -> Self {
        let server = world.resource::<AssetServer>();
        Self {
            images: ENTRY_DESIGNS
                .each_ref()
                .map(|entry| server.load(entry.image)),
        }
    }
}

pub(super) fn load_entry_art(
    assets: Res<GalleryAssets>,
    mut entries: Query<(&EntryArt, &mut ImageNode), Added<EntryArt>>,
) {
    for (art, mut image) in &mut entries {
        image.image = assets.images[art.0].clone();
    }
}
