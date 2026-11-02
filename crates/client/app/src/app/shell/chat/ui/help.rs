use super::controls::{side_button_tint, side_control_node};
use crate::app::runtime::UiAssets;
use crate::app::shell::UiAction;
use bevy::prelude::*;
use bevy::ui_widgets::Button;
use leocard_ui::UiComposer;

pub(crate) fn add_chat_help_button(
    commands: &mut UiComposer,
    chat_panel: Entity,
    action: UiAction,
    assets: &UiAssets,
) {
    commands.child(
        chat_panel,
        (
            Button,
            action,
            side_control_node(234.0),
            GlobalZIndex(2000),
            assets.tabletop.help_image(),
            side_button_tint(),
        ),
    );
}
