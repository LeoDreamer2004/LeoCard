use super::super::UiAction;
use crate::app::presentation::{TEXT, add_text, spawn_node};
use crate::app::runtime::UiAssets;
use bevy::ui_widgets::Button;
use bevy::{picking::Pickable, prelude::*};

pub(crate) fn add_page_back_title(
    commands: &mut Commands,
    parent: Entity,
    label: &str,
    action: UiAction,
    assets: &UiAssets,
) {
    let title = spawn_node(
        commands,
        parent,
        Node {
            align_items: AlignItems::Center,
            column_gap: px(12),
            ..default()
        },
        None,
    );
    commands.entity(title).insert((Button, action));
    let arrow = commands
        .spawn((
            Node {
                width: px(24),
                height: px(14),
                margin: UiRect::horizontal(px(6)),
                ..default()
            },
            ImageNode::new(assets.achievements.scroll_arrow.clone()),
            UiTransform::from_rotation(Rot2::degrees(-90.0)),
            Pickable::IGNORE,
        ))
        .id();
    commands.entity(title).add_child(arrow);
    add_text(commands, title, label, 31.0, TEXT, assets);
}
