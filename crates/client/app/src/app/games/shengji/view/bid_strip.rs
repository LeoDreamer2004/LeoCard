use super::super::{ShengjiAssets, ShengjiUiAction};
use super::bidding::shengji_declaration_candidate;
use crate::app::presentation::{TEXT, UiComposer, add_text, sliced_image, spawn_node};
use crate::app::runtime::UiAssets;
use crate::app::shell::UiAction;
use bevy::picking::Pickable;
use bevy::prelude::*;
use bevy::ui_widgets::Button;
use leocard_protocol::ShengjiSnapshot;
use leocard_shengji::ShengjiSuit;

const SEGMENT_WIDTH: f32 = 76.0;
const NG_SEGMENT_WIDTH: f32 = 56.0;
const SEGMENT_HEIGHT: f32 = 54.0;

pub(super) fn add_shengji_bid_strip(
    commands: &mut UiComposer,
    parent: Entity,
    game: &ShengjiSnapshot,
    bottom_copy: bool,
    assets: &UiAssets,
    game_assets: &ShengjiAssets,
) {
    let strip = spawn_node(
        commands,
        parent,
        Node {
            width: px(SEGMENT_WIDTH * 4.0 + NG_SEGMENT_WIDTH),
            height: px(SEGMENT_HEIGHT),
            flex_shrink: 0.0,
            align_items: AlignItems::Center,
            ..default()
        },
        None,
    );
    commands.entity(strip).insert(sliced_image(
        game_assets.bid_capsule.clone(),
        39.0,
        1.0 / 3.0,
    ));
    for suit in [
        Some(ShengjiSuit::Spade),
        Some(ShengjiSuit::Heart),
        Some(ShengjiSuit::Club),
        Some(ShengjiSuit::Diamond),
        None,
    ] {
        add_segment(
            commands,
            strip,
            game,
            suit,
            bottom_copy,
            assets,
            game_assets,
        );
    }
}

fn add_segment(
    commands: &mut UiComposer,
    parent: Entity,
    game: &ShengjiSnapshot,
    suit: Option<ShengjiSuit>,
    bottom_copy: bool,
    assets: &UiAssets,
    game_assets: &ShengjiAssets,
) {
    let cards = shengji_declaration_candidate(game, suit);
    let enabled = cards.is_some();
    let button = commands.keyed_child(
        parent,
        format!("bid:{suit:?}"),
        (
            Node {
                width: px(if suit.is_some() {
                    SEGMENT_WIDTH
                } else {
                    NG_SEGMENT_WIDTH
                }),
                height: px(SEGMENT_HEIGHT),
                flex_shrink: 0.0,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                column_gap: px(2),
                ..default()
            },
            if enabled {
                Pickable::default()
            } else {
                Pickable::IGNORE
            },
        ),
    );
    if let Some(cards) = cards {
        commands.entity(button).insert((
            Button,
            UiAction::Shengji(if bottom_copy {
                ShengjiUiAction::BottomCopy(cards)
            } else {
                ShengjiUiAction::Declare(cards)
            }),
        ));
    }
    if suit.is_some() {
        let divider = spawn_node(
            commands,
            button,
            Node {
                position_type: PositionType::Absolute,
                right: px(0),
                top: px(8),
                bottom: px(8),
                width: px(1),
                ..default()
            },
            Some(Color::srgba(0.72, 0.72, 0.76, 0.24)),
        );
        commands.entity(divider).insert(Pickable::IGNORE);
    }
    commands.child(
        button,
        (
            Node {
                width: px(32),
                height: px(32),
                flex_shrink: 0.0,
                ..default()
            },
            game_assets.bid_icon(suit, enabled),
            Pickable::IGNORE,
        ),
    );
    let Some(suit) = suit else { return };
    let count = game
        .your_hand
        .iter()
        .filter(|card| card.suit() == Some(suit))
        .count();
    let count = add_text(
        commands,
        button,
        count.to_string(),
        19.0,
        if enabled {
            TEXT
        } else {
            Color::srgb(0.54, 0.55, 0.60)
        },
        assets,
    );
    commands.entity(count).insert(Pickable::IGNORE);
}
