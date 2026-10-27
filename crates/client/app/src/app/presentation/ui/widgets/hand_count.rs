use crate::app::presentation::{TEXT, add_text};
use crate::app::runtime::UiAssets;
use bevy::picking::Pickable;
use bevy::prelude::*;
use leocard_ui::{UiComposer, spawn_node};

pub(crate) fn add_hand_count(
    commands: &mut UiComposer,
    parent: Entity,
    count: u16,
    assets: &UiAssets,
) {
    let fan = spawn_node(
        commands,
        parent,
        Node {
            width: px(28),
            height: px(24),
            flex_shrink: 0.0,
            position_type: PositionType::Relative,
            ..default()
        },
        None,
    );
    commands.entity(fan).insert(Pickable::IGNORE);
    for (left, top, rotation, layer) in [
        (2.0, 4.0, -0.27, 0),
        (8.0, 1.0, 0.0, 1),
        (14.0, 4.0, 0.27, 2),
    ] {
        commands.child(
            fan,
            (
                Node {
                    position_type: PositionType::Absolute,
                    left: px(left),
                    top: px(top),
                    width: px(12),
                    height: px(18),
                    ..default()
                },
                ImageNode::new(assets.playing_cards.card_back.clone()),
                UiTransform::from_rotation(Rot2::radians(rotation)),
                ZIndex(layer),
                Pickable::IGNORE,
            ),
        );
    }
    let number = add_text(commands, parent, count.to_string(), 14.0, TEXT, assets);
    commands.entity(number).insert((
        TextLayout::default().with_no_wrap(),
        TextShadow {
            offset: Vec2::new(1.0, 1.0),
            color: Color::BLACK.with_alpha(0.8),
        },
        Pickable::IGNORE,
    ));
}
