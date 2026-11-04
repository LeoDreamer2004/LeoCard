use super::lan::{LanDiscovery, LanRoomList, sync_lan_discovery};
use super::{
    dispatch_connection_actions, form::sync_connection_inputs, handle_avatar_drop,
    poll_avatar_picker, sync_avatar_images, view::GameGalleryPlugin,
};
use crate::app::runtime::ClientUpdateSet;
use crate::app::shell::{ModalAnimationSet, PageTransitionSet, UiActionSet, scroll_modal_content};
use bevy::prelude::*;
use leocard_ui::TextInputSet;

pub(crate) struct ConnectionPlugin;

impl Plugin for ConnectionPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            super::view::animate_home_overlays
                .in_set(ClientUpdateSet::Animate)
                .after(PageTransitionSet)
                .before(ModalAnimationSet::Visuals),
        );
        app.init_resource::<LanDiscovery>()
            .add_plugins(GameGalleryPlugin)
            .add_systems(
                Update,
                dispatch_connection_actions
                    .in_set(UiActionSet)
                    .before(ClientUpdateSet::Sync),
            )
            .add_systems(
                Update,
                (handle_avatar_drop, poll_avatar_picker)
                    .chain()
                    .in_set(ClientUpdateSet::Input),
            )
            .add_systems(
                Update,
                (
                    sync_avatar_images,
                    sync_lan_discovery,
                    super::state::track_recent_game,
                )
                    .in_set(ClientUpdateSet::Sync),
            )
            .add_systems(
                Update,
                (
                    scroll_modal_content::<LanRoomList>,
                    scroll_modal_content::<super::view::GameLibrary>,
                )
                    .in_set(ClientUpdateSet::Input),
            )
            .add_systems(
                PostUpdate,
                sync_connection_inputs.in_set(TextInputSet::Publish),
            );
    }
}
