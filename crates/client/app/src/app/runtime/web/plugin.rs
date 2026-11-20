use super::lifecycle::{mark_started, restore_room, sync_active_room, sync_image_drop};
use super::text_input::{apply_browser_edits, publish_browser_editors};
use super::textures::normalize_browser_images;
use bevy::prelude::*;
use bevy::ui::UiSystems;

pub(in crate::app::runtime) struct WebRuntimePlugin;

impl Plugin for WebRuntimePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(PostStartup, (mark_started, restore_room));
        app.add_systems(Update, sync_active_room);
        app.add_systems(PreUpdate, sync_image_drop);
        app.add_systems(PostUpdate, normalize_browser_images);
        app.add_systems(PreUpdate, apply_browser_edits);
        app.add_systems(
            PostUpdate,
            publish_browser_editors.after(UiSystems::PostLayout),
        );
    }
}
