use super::{
    BackgroundAssets, BackgroundMaterial, animate_background, setup_background,
    sync_background_visibility,
};
use crate::app::runtime::ClientUpdateSet;
use bevy::prelude::*;
use bevy::ui_render::UiMaterialPlugin;

pub(crate) struct AmbientBackgroundPlugin;

impl Plugin for AmbientBackgroundPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(UiMaterialPlugin::<BackgroundMaterial>::default())
            .init_resource::<BackgroundAssets>()
            .add_systems(Startup, setup_background)
            .add_systems(
                Update,
                (sync_background_visibility, animate_background).in_set(ClientUpdateSet::Animate),
            );
    }
}
