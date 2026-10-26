use super::lan::{LanDiscovery, LanRoomList, sync_lan_discovery};
use super::{
    dispatch_connection_actions, form::sync_connection_inputs, handle_avatar_drop,
    poll_avatar_picker, publish_avatar, sync_avatar_images, view::GameGalleryPlugin,
};
use crate::app::runtime::ClientUpdateSet;
use crate::app::shell::{UiActionSet, scroll_modal_content};
use bevy::prelude::*;
use leocard_ui::TextInputSet;

pub(crate) struct ConnectionPlugin;

impl Plugin for ConnectionPlugin {
    fn build(&self, app: &mut App) {
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
                (sync_avatar_images, publish_avatar, sync_lan_discovery)
                    .in_set(ClientUpdateSet::Sync),
            )
            .add_systems(
                Update,
                scroll_modal_content::<LanRoomList>.in_set(ClientUpdateSet::Input),
            )
            .add_systems(
                PostUpdate,
                sync_connection_inputs.in_set(TextInputSet::Publish),
            );
    }
}
