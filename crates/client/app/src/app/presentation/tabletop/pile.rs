use super::{TableCard, TableCardMaterial};
use crate::app::presentation::{TEXT, add_text};
use crate::app::runtime::UiAssets;
use bevy::prelude::*;
use leocard_ui::{UiComposer, spawn_node};

pub(crate) fn add_playing_card_pile(
    commands: &mut UiComposer,
    parent: Entity,
    count: usize,
    assets: &UiAssets,
) -> Entity {
    add_card_pile(commands, parent, count, assets, None)
}

pub(crate) fn add_table_card_pile(
    commands: &mut UiComposer,
    parent: Entity,
    count: usize,
    assets: &UiAssets,
    materials: &mut Assets<TableCardMaterial>,
) -> Entity {
    add_card_pile(commands, parent, count, assets, Some(materials))
}

pub(crate) fn table_card_pile_top_center() -> Vec2 {
    Vec2::new(4.0 * 1.7 + 43.0 * 0.5, 58.0 * 0.5)
}

fn add_card_pile(
    commands: &mut UiComposer,
    parent: Entity,
    count: usize,
    assets: &UiAssets,
    mut materials: Option<&mut Assets<TableCardMaterial>>,
) -> Entity {
    let projected = materials.is_some();
    let pile = spawn_node(
        commands,
        parent,
        Node {
            width: px(if projected { 50.0 } else { 44.0 }),
            height: px(if projected { 70.0 } else { 62.0 }),
            position_type: PositionType::Relative,
            ..default()
        },
        None,
    );
    for layer in 0..5 {
        if let Some(materials) = materials.as_deref_mut() {
            let card = TableCard {
                texture: assets.playing_cards.card_back.clone(),
                size: Vec2::new(43.0, 58.0),
                opacity: 1.0,
            }
            .spawn(
                commands,
                pile,
                Node {
                    position_type: PositionType::Absolute,
                    left: px(layer as f32 * 1.7),
                    top: px((4 - layer) as f32),
                    ..default()
                },
                materials,
            );
            commands.entity(card).insert(ZIndex(layer));
            continue;
        }
        commands.child(
            pile,
            (
                Node {
                    position_type: PositionType::Absolute,
                    left: px(layer as f32 * 1.7),
                    top: px((4 - layer) as f32 * 1.0),
                    width: px(39),
                    height: px(54),
                    border: UiRect::all(px(1)),
                    border_radius: BorderRadius::all(px(4)),
                    ..default()
                },
                ImageNode::new(assets.playing_cards.card_back.clone()),
                BorderColor::all(TEXT.with_alpha(0.52)),
                ZIndex(layer),
            ),
        );
    }
    let counter = spawn_node(
        commands,
        pile,
        Node {
            position_type: PositionType::Absolute,
            left: px(if projected { 10.0 } else { 7.0 }),
            top: px(if projected { 34.0 } else { 27.0 }),
            width: px(29),
            height: px(21),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            border_radius: BorderRadius::all(px(6)),
            ..default()
        },
        Some(Color::BLACK.with_alpha(0.68)),
    );
    commands.entity(counter).insert(ZIndex(8));
    add_text(commands, counter, count.to_string(), 12.0, TEXT, assets);
    pile
}
