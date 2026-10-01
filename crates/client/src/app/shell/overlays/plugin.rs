use super::{PlayErrorToast, animate_play_error_popup, sync_play_error_toast};
use crate::app::runtime::ClientUpdateSet;
use bevy::prelude::*;
pub(crate) struct OverlaysPlugin;
impl Plugin for OverlaysPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PlayErrorToast>()
            .add_systems(Update, sync_play_error_toast.in_set(ClientUpdateSet::Sync))
            .add_systems(
                Update,
                animate_play_error_popup.in_set(ClientUpdateSet::Animate),
            );
    }
}
