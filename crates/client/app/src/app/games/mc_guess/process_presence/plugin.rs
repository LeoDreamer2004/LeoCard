#[cfg(not(target_os = "android"))]
use super::monitor::{McProcessMonitor, monitor_mc_process};
use super::notifications::{McProcessNotices, show_process_notices};
use crate::app::runtime::ClientUpdateSet;
use crate::app::shell::sync_play_error_toast;
use bevy::prelude::*;

pub(in super::super) struct McProcessPresencePlugin;

impl Plugin for McProcessPresencePlugin {
    fn build(&self, app: &mut App) {
        #[cfg(not(target_os = "android"))]
        app.init_resource::<McProcessMonitor>()
            .add_systems(Update, monitor_mc_process.in_set(ClientUpdateSet::Sync));
        app.init_resource::<McProcessNotices>().add_systems(
            Update,
            show_process_notices
                .after(sync_play_error_toast)
                .in_set(ClientUpdateSet::Sync),
        );
    }
}
