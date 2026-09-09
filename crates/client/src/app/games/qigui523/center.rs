use super::sort_cards_high_to_low;
use crate::app::presentation::CardSize;
use crate::app::presentation::{HEADER_BG, MUTED, TEXT, add_card_image, add_text, spawn_node};
use crate::app::runtime::UiAssets;
use bevy::prelude::*;
use leocard_protocol::PublicPlayRecord;
use leocard_protocol::QiGui523Snapshot;

pub(super) fn add_draw_pile(
    commands: &mut Commands,
    parent: Entity,
    count: usize,
    assets: &UiAssets,
) {
    let layers = if count == 0 {
        0
    } else {
        count.div_ceil(14).clamp(1, 8)
    };
    let pile = spawn_node(
        commands,
        parent,
        Node {
            width: px(48.0 + layers as f32 * 2.0),
            height: px(64.0 + layers as f32 * 1.5),
            position_type: PositionType::Relative,
            margin: UiRect::bottom(px(3)),
            ..default()
        },
        None,
    );
    if layers == 0 {
        let empty = spawn_node(
            commands,
            pile,
            Node {
                position_type: PositionType::Absolute,
                left: px(1),
                top: px(1),
                width: px(46),
                height: px(62),
                border: UiRect::all(px(2)),
                border_radius: BorderRadius::all(px(5)),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
            Some(HEADER_BG.with_alpha(0.36)),
        );
        commands
            .entity(empty)
            .insert(BorderColor::all(MUTED.with_alpha(0.45)));
    } else {
        for layer in 0..layers {
            let card = commands
                .spawn((
                    Node {
                        position_type: PositionType::Absolute,
                        left: px(layer as f32 * 2.0),
                        top: px((layers - layer - 1) as f32 * 1.5),
                        width: px(46),
                        height: px(62),
                        border: UiRect::all(px(1)),
                        border_radius: BorderRadius::all(px(5)),
                        ..default()
                    },
                    ImageNode::new(assets.playing_cards.card_back.clone()),
                    BorderColor::all(TEXT.with_alpha(0.55)),
                    BoxShadow::new(Color::BLACK.with_alpha(0.28), px(1), px(2), px(0), px(3)),
                    ZIndex(layer as i32),
                ))
                .id();
            commands.entity(pile).add_child(card);
        }
    }

    let counter = spawn_node(
        commands,
        pile,
        Node {
            position_type: PositionType::Absolute,
            left: percent(50),
            top: percent(50),
            width: px(34),
            height: px(28),
            margin: UiRect {
                left: px(-17),
                top: px(-14),
                ..default()
            },
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            border_radius: BorderRadius::all(px(7)),
            ..default()
        },
        Some(Color::BLACK.with_alpha(0.72)),
    );
    commands.entity(counter).insert(ZIndex(layers as i32 + 1));
    add_text(commands, counter, count.to_string(), 18.0, TEXT, assets);
}

pub(super) fn add_table_score_cards(
    commands: &mut Commands,
    parent: Entity,
    game: &QiGui523Snapshot,
    assets: &UiAssets,
) {
    let Some(trick) = game.trick.as_ref() else {
        return;
    };
    let mut cards = trick
        .records
        .iter()
        .flat_map(|record| match record {
            PublicPlayRecord::Played { play, .. } => play.cards.as_slice(),
            PublicPlayRecord::Passed { .. } => &[],
        })
        .copied()
        .filter(|card| card.score() > 0)
        .collect::<Vec<_>>();
    if cards.is_empty() {
        return;
    }
    sort_cards_high_to_low(&mut cards);
    let hand = spawn_node(
        commands,
        parent,
        Node {
            height: px(CardSize::TableScore.dimensions().1),
            flex_direction: FlexDirection::Row,
            flex_wrap: FlexWrap::NoWrap,
            align_items: AlignItems::Center,
            ..default()
        },
        None,
    );
    let last_card = cards.len().saturating_sub(1);
    for (index, card) in cards.into_iter().enumerate() {
        add_card_image(
            commands,
            hand,
            card,
            CardSize::TableScore,
            index,
            index == last_card,
            false,
            assets,
        );
    }
}
