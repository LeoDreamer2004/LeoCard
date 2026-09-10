use super::{
    ChatPanelState, actions::dispatch_chat_actions, animate_chat_bubbles, animate_chat_panel,
    scroll_chat_menus, sync_chat_messages, sync_chat_panel_text,
};
use crate::app::runtime::ClientUpdateSet;
use crate::app::shell::UiActionSet;
use bevy::prelude::*;

pub(crate) struct ChatPlugin;

impl Plugin for ChatPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(ChatPanelState::default())
            .add_systems(Update, dispatch_chat_actions.in_set(UiActionSet))
            .add_systems(
                Update,
                (sync_chat_messages, sync_chat_panel_text, scroll_chat_menus)
                    .in_set(ClientUpdateSet::Sync),
            )
            .add_systems(
                Update,
                (animate_chat_bubbles, animate_chat_panel).in_set(ClientUpdateSet::Animate),
            );
    }
}
