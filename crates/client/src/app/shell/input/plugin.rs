use super::{UiZoom, handle_text_input, sync_ime_enabled, update_ui_scale};
use crate::app::runtime::ClientUpdateSet;
use bevy::prelude::*;
pub(crate) struct InputPlugin;
impl Plugin for InputPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<UiZoom>().add_systems(
            Update,
            (sync_ime_enabled, update_ui_scale, handle_text_input)
                .chain()
                .in_set(ClientUpdateSet::Input),
        );
    }
}
