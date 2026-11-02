use super::super::{ChatPanel, ChatPanelState, ChatToggleIcon};
use bevy::prelude::*;
use leocard_ui::DrawerSlide;

pub(crate) fn sync_chat_panel_controls(
    chat: Res<ChatPanelState>,
    mut panels: Query<&mut DrawerSlide, With<ChatPanel>>,
    mut icons: Query<&mut ImageNode, With<ChatToggleIcon>>,
) {
    for mut icon in &mut icons {
        if icon.flip_x != chat.open {
            icon.flip_x = chat.open;
        }
    }
    for mut slide in &mut panels {
        // Retained outgoing and incoming panels share one continuous motion.
        slide.clone_from(&chat.drawer);
        slide.expanded = chat.open;
    }
}

/// Keep presentation progress when the whole chat subtree is recreated.
pub(crate) fn retain_chat_drawer_motion(
    panels: Query<&DrawerSlide, With<ChatPanel>>,
    mut chat: ResMut<ChatPanelState>,
) {
    if let Some(slide) = panels.iter().next()
        && chat.drawer != *slide
    {
        chat.drawer = slide.clone();
    }
}
