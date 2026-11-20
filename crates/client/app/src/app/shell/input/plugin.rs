use super::back::handle_escape;
use crate::app::runtime::ClientUpdateSet;
#[cfg(feature = "android")]
use crate::app::shell::LobbyEntranceState;
use crate::app::shell::UiActionSet;
use bevy::prelude::*;

pub(crate) struct InputPlugin;

impl Plugin for InputPlugin {
    fn build(&self, app: &mut App) {
        #[cfg(feature = "web")]
        app.add_systems(PreUpdate, super::web::handle_back);
        #[cfg(feature = "android")]
        app.add_systems(
            PreUpdate,
            super::android::handle_back
                .run_if(|entrance: Res<LobbyEntranceState>| !entrance.active()),
        );
        app.add_systems(
            Update,
            handle_escape
                .in_set(ClientUpdateSet::Input)
                .before(UiActionSet),
        );
    }
}
