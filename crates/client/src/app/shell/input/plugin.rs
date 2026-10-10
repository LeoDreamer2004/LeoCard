use super::{UiZoom, update_ui_scale};
use crate::app::runtime::ClientUpdateSet;
use bevy::prelude::*;

pub(crate) struct InputPlugin;

impl Plugin for InputPlugin {
    fn build(&self, app: &mut App) {
        #[cfg(target_os = "android")]
        app.add_systems(PreUpdate, super::android::handle_back);
        app.init_resource::<UiZoom>()
            .add_systems(Update, update_ui_scale.in_set(ClientUpdateSet::Input));
    }
}
