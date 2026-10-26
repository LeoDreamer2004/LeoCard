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
            Node {
                position_type: PositionType::Absolute,
                left: px(-32),
                top: px(234),
                width: px(32),
                height: px(32),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
            GlobalZIndex(2000),
            ImageNode::new(assets.home.help_question.clone()),
        ),
    );
}
