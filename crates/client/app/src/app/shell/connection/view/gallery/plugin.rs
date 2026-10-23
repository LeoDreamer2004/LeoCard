use super::{GalleryAssets, animate_entries, load_entry_art};
use crate::app::runtime::ClientUpdateSet;
use crate::app::shell::PageTransitionSet;
use bevy::prelude::*;

pub(in crate::app::shell::connection) struct GameGalleryPlugin;

impl Plugin for GameGalleryPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<GalleryAssets>().add_systems(
            Update,
            (load_entry_art, animate_entries)
                .chain()
                .after(PageTransitionSet)
                .in_set(ClientUpdateSet::Animate),
        );
    }
}
