use super::assets::BackgroundAssets;
use super::material::GuessBackgroundMaterial;
use super::scene::{GuessBackgroundCanvas, GuessBackgroundPage};
use super::state::{BackgroundPlayback, PendingPanorama, matched_version};
use crate::app::runtime::ClientResource;
use bevy::asset::RenderAssetUsages;
use bevy::image::{ImageArrayLayout, ImageLoaderSettings, ImageSampler};
use bevy::prelude::*;
use bevy::render::render_resource::{TextureViewDescriptor, TextureViewDimension};
use std::f32::consts;

const TRANSITION_SECONDS: f32 = 1.5;
const SPIN_DEGREES_PER_SECOND: f32 = 2.0;

pub(super) fn sync_background(
    client: Option<Res<ClientResource>>,
    pages: Query<(), With<GuessBackgroundPage>>,
    server: Res<AssetServer>,
    assets: Res<BackgroundAssets>,
    mut playback: ResMut<BackgroundPlayback>,
    mut materials: ResMut<Assets<GuessBackgroundMaterial>>,
) {
    let game = client
        .as_deref()
        .and_then(|client| client.0.model().minecraft_guess_game());
    // Retain the old scene until the exit transition removes its page.
    if game.is_none() && !pages.is_empty() {
        return;
    }
    let round = game.map(|game| (game.match_id, game.progress.game_index));
    if playback.round != round {
        playback.begin_round(round);
        if round.is_none() {
            let mut material = materials
                .get_mut(&assets.material)
                .expect("background material");
            material.panorama = None;
            material.params.y = 0.0;
        }
    }
    if playback.needs_panorama()
        && let Some(version) = game.and_then(|game| matched_version(&game.round))
    {
        playback.pending_panorama = Some(PendingPanorama {
            image: server
                .load_builder()
                .with_settings(|settings: &mut ImageLoaderSettings| {
                    settings.array_layout = Some(ImageArrayLayout::RowCount { rows: 6 });
                    settings.sampler = ImageSampler::linear();
                })
                .load(assets.panorama_path(version)),
            ready: false,
        });
    }
}

pub(super) fn animate_background(
    time: Res<Time>,
    pages: Query<(), With<GuessBackgroundPage>>,
    mut canvases: Query<(&mut Visibility, &ComputedNode), With<GuessBackgroundCanvas>>,
    assets: Res<BackgroundAssets>,
    mut playback: ResMut<BackgroundPlayback>,
    mut images: ResMut<Assets<Image>>,
    mut materials: ResMut<Assets<GuessBackgroundMaterial>>,
) {
    let Some((mut visibility, node)) = canvases.iter_mut().next() else {
        return;
    };
    *visibility = if pages.is_empty() {
        Visibility::Hidden
    } else {
        Visibility::Visible
    };
    if pages.is_empty() {
        return;
    }
    let mut material = materials
        .get_mut(&assets.material)
        .expect("background material");
    let current_panorama = playback.panorama.clone();
    if let Some(panorama) = playback.pending_panorama.as_mut()
        && !panorama.ready
    {
        if current_panorama.as_ref() == Some(&panorama.image) {
            // The current texture is already on the GPU and may have left Assets<Image>.
            panorama.ready = true;
        } else if let Some(mut image) = images.get_mut(&panorama.image) {
            image.texture_view_descriptor = Some(TextureViewDescriptor {
                dimension: Some(TextureViewDimension::Cube),
                ..default()
            });
            image.asset_usage = RenderAssetUsages::RENDER_WORLD;
            panorama.ready = true;
        }
    }
    playback.advance(time.delta_secs(), TRANSITION_SECONDS);
    material.panorama = playback.panorama.clone();
    material.params = Vec4::new(
        (playback.seconds * SPIN_DEGREES_PER_SECOND)
            .to_radians()
            .rem_euclid(consts::TAU),
        playback.transition,
        node.inverse_scale_factor(),
        0.0,
    );
}
