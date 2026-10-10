use super::AppearancePreferences;
use bevy::audio::{GlobalVolume, Volume};
use bevy::prelude::*;
use bevy::render::RenderPlugin;
use bevy::render::settings::{InstanceFlags, RenderCreation, WgpuSettings};
use leocard_client::platform::audio_active;

pub(super) fn render_plugin() -> RenderPlugin {
    let mut settings = WgpuSettings::default();
    // ARM64 translation in the Android emulator crashes in vkSetDebugUtilsObjectNameEXT.
    // Keep validation, but do not ask Android drivers to attach debug labels to GPU objects.
    settings.instance_flags.remove(InstanceFlags::DEBUG);
    RenderPlugin {
        render_creation: RenderCreation::Automatic(Box::new(settings)),
        ..default()
    }
}

pub(super) fn sync_audio_focus(
    preferences: Res<AppearancePreferences>,
    mut volume: ResMut<GlobalVolume>,
) {
    let desired = Volume::Linear(if audio_active() {
        preferences.audio_volume
    } else {
        0.0
    });
    if volume.volume != desired {
        volume.volume = desired;
    }
}
