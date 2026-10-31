use super::super::{ChatPanel, ChatPanelState, ChatToggleIcon};
use crate::app::runtime::UiAssets;
use bevy::picking::hover::PickingInteraction;
use bevy::prelude::*;
use leocard_ui::DrawerSlide;

pub(crate) fn sync_chat_panel_controls(
    assets: Res<UiAssets>,
    chat: Res<ChatPanelState>,
    mut panels: Query<&mut DrawerSlide, With<ChatPanel>>,
    mut icons: Query<(&PickingInteraction, &mut ImageNode), With<ChatToggleIcon>>,
) {
    for (interaction, mut icon) in &mut icons {
        let expected = match (chat.open, *interaction != PickingInteraction::None) {
            (false, false) => &assets.home.rule_left,
            (false, true) => &assets.home.rule_left_highlighted,
            (true, false) => &assets.home.rule_right,
            (true, true) => &assets.home.rule_right_highlighted,
        };
        if icon.image != *expected {
            icon.image = expected.clone();
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
