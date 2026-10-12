use super::assets::BackgroundAssets;
use super::material::GuessBackgroundMaterial;
use super::motion::{animate_background, sync_background};
use super::scene::setup_background;
use super::state::BackgroundPlayback;
use crate::app::runtime::ClientUpdateSet;
use bevy::prelude::*;
use bevy::ui_render::UiMaterialPlugin;

pub(in super::super) struct GuessBackgroundPlugin;

impl Plugin for GuessBackgroundPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(UiMaterialPlugin::<GuessBackgroundMaterial>::default())
            .init_resource::<BackgroundAssets>()
            .init_resource::<BackgroundPlayback>()
            .add_systems(Startup, setup_background)
            .add_systems(
                Update,
                (sync_background, animate_background)
                    .chain()
                    .in_set(ClientUpdateSet::Animate),
            );
    }
}
