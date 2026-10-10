use super::super::sync_shengji_presentation;
use super::assets::load_shengji_voice_assets;
use super::playback::play_shengji_voices;
use super::voices::{ShengjiVoiceState, sync_shengji_voices};
use crate::app::runtime::ClientUpdateSet;
use bevy::prelude::*;

pub(in crate::app::games::shengji) struct ShengjiAudioPlugin;

impl Plugin for ShengjiAudioPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ShengjiVoiceState>()
            .add_systems(Startup, load_shengji_voice_assets)
            .add_systems(
                Update,
                sync_shengji_voices
                    .after(sync_shengji_presentation)
                    .in_set(ClientUpdateSet::Sync),
            )
            .add_systems(Update, play_shengji_voices.in_set(ClientUpdateSet::Animate));
    }
}
