use super::form::{submit_room_inputs, sync_connection_inputs};
use super::lan::{LanDiscovery, sync_lan_discovery};
use super::rooms::{
    JoinRoomList, RoomDirectory, advance_room_dialog, animate_room_signals, load_room_assets,
    sync_room_directory,
};
use super::{
    dispatch_connection_actions, handle_avatar_drop, poll_avatar_picker, sync_avatar_images,
    view::GameGalleryPlugin,
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
            .init_resource::<RoomDirectory>()
            .add_systems(Startup, load_room_assets)
            .add_systems(
                Update,
                advance_room_dialog.in_set(ModalAnimationSet::Progress),
            )
            .add_systems(
                Update,
                animate_room_signals.in_set(ClientUpdateSet::Animate),
            )
            .add_plugins(GameGalleryPlugin)
            .add_systems(
                Update,
                dispatch_connection_actions
                    .in_set(UiActionSet)
                    .before(ClientUpdateSet::Sync),
            )
            .add_systems(
                Update,
                (handle_avatar_drop, poll_avatar_picker, submit_room_inputs)
                    .chain()
                    .in_set(ClientUpdateSet::Input),
            )
            .add_systems(
                Update,
                (
                    sync_avatar_images,
                    sync_lan_discovery,
                    sync_room_directory,
                    super::state::track_recent_game,
                )
                    .chain()
                    .in_set(ClientUpdateSet::Sync),
            )
            .add_systems(
                Update,
                (
                    scroll_modal_content::<JoinRoomList>,
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
