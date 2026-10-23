#[cfg(not(target_os = "android"))]
use super::monitor::{MinecraftProcessMonitor, monitor_minecraft_process};
use super::notifications::{MinecraftProcessNotices, show_process_notices};
use crate::app::runtime::ClientUpdateSet;
use crate::app::shell::sync_play_error_toast;
use bevy::prelude::*;

pub(in super::super) struct MinecraftProcessPresencePlugin;

impl Plugin for MinecraftProcessPresencePlugin {
    fn build(&self, app: &mut App) {
        #[cfg(not(target_os = "android"))]
        app.init_resource::<MinecraftProcessMonitor>().add_systems(
            Update,
            monitor_minecraft_process.in_set(ClientUpdateSet::Sync),
        );
        app.init_resource::<MinecraftProcessNotices>().add_systems(
            Update,
            show_process_notices
                .after(sync_play_error_toast)
                .in_set(ClientUpdateSet::Sync),
        );
    }
}
