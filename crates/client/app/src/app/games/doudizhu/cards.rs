use crate::app::presentation::{CardSize, add_card_image};
use crate::app::runtime::{CardFace, CardRank, CardSuit, UiAssets};
use bevy::prelude::*;
use leocard_doudizhu::Card;
use leocard_ui::{UiComposer, spawn_node};

impl From<Card> for CardFace {
    fn from(card: Card) -> Self {
        let rank = match card.rank() {
            0 => CardRank::Three,
            1 => CardRank::Four,
            2 => CardRank::Five,
            3 => CardRank::Six,
            4 => CardRank::Seven,
            5 => CardRank::Eight,
            6 => CardRank::Nine,
            7 => CardRank::Ten,
            8 => CardRank::Jack,
            9 => CardRank::Queen,
            10 => CardRank::King,
            11 => CardRank::Ace,
            12 => CardRank::Two,
            13 => return Self::SmallJoker,
            _ => return Self::BigJoker,
        };
        let suit = match card.suit() {
            0 => CardSuit::Diamond,
            1 => CardSuit::Club,
            2 => CardSuit::Heart,
            _ => CardSuit::Spade,
        };
        Self::Suited { rank, suit }
    }
}

pub(super) fn card_row(
    commands: &mut UiComposer,
    parent: Entity,
    cards: &[Card],
    size: CardSize,
    assets: &UiAssets,
) {
    card_row_aligned(commands, parent, cards, size, assets, AlignItems::Center);
}

pub(super) fn card_row_aligned(
    commands: &mut UiComposer,
    parent: Entity,
    cards: &[Card],
    size: CardSize,
    assets: &UiAssets,
    alignment: AlignItems,
) {
    let column = spawn_node(
        commands,
        parent,
        Node {
            flex_direction: FlexDirection::Column,
            width: percent(100),
            align_items: alignment,
            row_gap: px(3),
            ..default()
        },
        None,
    );
    let mut sorted = cards.to_vec();
    sorted.sort_by_key(|c| (c.rank(), c.0));
    sorted.reverse();
    let per_row = match size {
        CardSize::FinishedHand => 8,
        _ => 14,
    };
    for chunk in sorted.chunks(per_row) {
        let row = spawn_node(
            commands,
            column,
            Node {
                width: px(size.dimensions().0
                    + chunk.len().saturating_sub(1) as f32
                        * if matches!(size, CardSize::FinishedHand) {
                            14.4
                        } else {
                            27.0
                        }),
                flex_shrink: 0.0,
                justify_content: JustifyContent::Center,
                flex_wrap: FlexWrap::NoWrap,
                ..default()
            },
            None,
        );
        for (i, card) in chunk.iter().enumerate() {
            add_card_image(
                commands,
                row,
                *card,
                size,
                i,
                i + 1 == chunk.len(),
                false,
                assets,
            );
        }
    }
}
