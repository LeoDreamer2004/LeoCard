use super::{
    ChatPanelState,
    actions::dispatch_chat_actions,
    input::{focus_chat_input, sync_chat_input},
    retain_chat_drawer_motion, scroll_chat_menus, sync_chat_bubble_anchors, sync_chat_messages,
    sync_chat_panel_controls, sync_chat_panel_text,
};
use crate::app::runtime::ClientUpdateSet;
use crate::app::shell::UiActionSet;
use bevy::prelude::*;
use leocard_ui::{TextInputSet, animate_bubbles, animate_drawers};

pub(crate) struct ChatPlugin;

impl Plugin for ChatPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(focus_chat_input)
            .insert_resource(ChatPanelState::default())
            .add_systems(PostUpdate, sync_chat_input.in_set(TextInputSet::Publish))
            .add_systems(Update, dispatch_chat_actions.in_set(UiActionSet))
            .add_systems(
                Update,
                (sync_chat_messages, sync_chat_panel_text, scroll_chat_menus)
                    .in_set(ClientUpdateSet::Sync),
            )
            .add_systems(
                Update,
                (
                    sync_chat_bubble_anchors.before(animate_bubbles),
                    sync_chat_panel_controls.before(animate_drawers),
                    retain_chat_drawer_motion.after(animate_drawers),
                )
                    .in_set(ClientUpdateSet::Animate),
            );
    }
}
