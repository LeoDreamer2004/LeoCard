use crate::app::presentation::{TEXT, UiComposer, add_text, spawn_node};
use crate::app::runtime::UiAssets;
use bevy::prelude::*;

pub(crate) fn add_playing_card_pile(
    commands: &mut UiComposer,
    parent: Entity,
    count: usize,
    assets: &UiAssets,
) -> Entity {
    let pile = spawn_node(
        commands,
        parent,
        Node {
            width: px(44),
            height: px(62),
            position_type: PositionType::Relative,
            ..default()
        },
        None,
    );
    for layer in 0..5 {
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
            left: px(7),
            top: px(27),
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
