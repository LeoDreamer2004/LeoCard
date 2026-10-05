use crate::app::runtime::UiAssets;
use crate::app::shell::UiAction;
use bevy::prelude::*;

pub(crate) fn add_chat_help_button(
    commands: &mut Commands,
    chat_panel: Entity,
    action: UiAction,
    assets: &UiAssets,
) {
    let button = commands
        .spawn((
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
        ))
        .id();
    commands.entity(chat_panel).add_child(button);
}
