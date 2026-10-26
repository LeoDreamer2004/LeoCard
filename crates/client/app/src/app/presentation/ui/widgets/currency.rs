use crate::app::presentation::{ACCENT, add_text};
use crate::app::runtime::UiAssets;
use bevy::{picking::Pickable, prelude::*};
use leocard_ui::{UiComposer, spawn_node};

pub(crate) fn add_coin_balance(
    commands: &mut UiComposer,
    parent: Entity,
    coins: u32,
    icon_size: f32,
    font_size: f32,
    assets: &UiAssets,
) -> Entity {
    let row = spawn_node(
        commands,
        parent,
        Node {
            align_items: AlignItems::Center,
            column_gap: px(5),
            ..default()
        },
        None,
    );
    commands.child(
        row,
        (
            Node {
                width: px(icon_size),
                height: px(icon_size),
                ..default()
            },
            ImageNode::new(assets.shop.coin.clone()),
            Pickable::IGNORE,
        ),
    );
    add_text(commands, row, coins.to_string(), font_size, ACCENT, assets);
    row
}
