use super::{
    AvatarImages, AvatarPicker, PageErrorState, PreferenceResources, TableAppearance,
    TableFeltPicker, asset_file_path, configure_runtime_asset_source, load_ui_assets, poll_network,
    set_app_window_icon, setup_camera,
};
use crate::app::presentation::{DESIGN_HEIGHT, DESIGN_WIDTH, TABLE_BG};
use bevy::asset::AssetPlugin;
use bevy::audio::{GlobalVolume, Volume};
use bevy::log::{DEFAULT_FILTER, LogPlugin};
use bevy::prelude::*;
use bevy::window::WindowResizeConstraints;
use leocard_client::{LocalPlayerProfile, PlayerIdentity, PlayerRatingProfile};
use leocard_protocol::PlayerGameProfiles;
use std::collections::HashSet;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, SystemSet)]
pub(crate) enum ClientUpdateSet {
    Input,
    Network,
    Sync,
    Rebuild,
    Animate,
}

pub(super) struct RuntimePlugin;

impl Plugin for RuntimePlugin {
    fn build(&self, app: &mut App) {
        let preferences = PreferenceResources::load();
        let mut page_error = PageErrorState::default();
        let profile = LocalPlayerProfile::load_or_create().unwrap_or_else(|error| {
            page_error.error = Some(error);
            LocalPlayerProfile {
                identity: PlayerIdentity::generate()
                    .expect("the operating system provides entropy"),
                rating: PlayerRatingProfile {
                    reference_points: 0,
                    completed_games: 0,
                    applied_matches: HashSet::new(),
                    last_change: None,
                },
                game_profiles: PlayerGameProfiles::default(),
            }
        });
        let audio_volume = preferences.appearance.audio_volume;

        configure_runtime_asset_source(app);
        app.insert_resource(ClearColor(TABLE_BG))
            .insert_resource(preferences.connection)
            .insert_resource(preferences.appearance)
            .insert_resource(preferences.host_rules)
            .insert_resource(page_error)
            .insert_resource(GlobalVolume::new(Volume::Linear(audio_volume)))
            .insert_resource(profile)
            .insert_resource(AvatarImages::default())
            .insert_resource(AvatarPicker::default())
            .insert_resource(TableFeltPicker::default())
            .insert_resource(TableAppearance::default())
            .add_plugins(
                DefaultPlugins
                    .set(LogPlugin {
                        filter: format!("{DEFAULT_FILTER}icu_provider::error=error"),
                        ..default()
                    })
                    .set(AssetPlugin {
                        file_path: asset_file_path(),
                        ..default()
                    })
                    .set(WindowPlugin {
                        primary_window: Some(Window {
                            title: "LeoCard".to_owned(),
                            resolution: (DESIGN_WIDTH as u32, DESIGN_HEIGHT as u32).into(),
                            resize_constraints: WindowResizeConstraints {
                                min_width: 640.0,
                                min_height: 400.0,
                                ..default()
                            },
                            resizable: true,
                            ..default()
                        }),
                        ..default()
                    }),
            )
            .configure_sets(
                Update,
                (
                    ClientUpdateSet::Input,
                    ClientUpdateSet::Network,
                    ClientUpdateSet::Sync,
                    ClientUpdateSet::Rebuild,
                    ClientUpdateSet::Animate,
                )
                    .chain(),
            )
            .add_systems(Startup, (setup_camera, load_ui_assets))
            .add_systems(Update, set_app_window_icon)
            .add_systems(Update, poll_network.in_set(ClientUpdateSet::Network));
    }
}
