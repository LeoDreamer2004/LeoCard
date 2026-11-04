use crate::app::presentation::{CardSize, add_card_image};
use crate::app::runtime::{CardFace, CardRank, CardSuit, UiAssets};
use bevy::prelude::*;
use leocard_guandan::{GuandanCard, GuandanSuit};
use leocard_ui::{UiComposer, spawn_node};

impl From<GuandanCard> for CardFace {
    fn from(card: GuandanCard) -> Self {
        let rank = match card.rank {
            2 => CardRank::Two,
            3 => CardRank::Three,
            4 => CardRank::Four,
            5 => CardRank::Five,
            6 => CardRank::Six,
            7 => CardRank::Seven,
            8 => CardRank::Eight,
            9 => CardRank::Nine,
            10 => CardRank::Ten,
            11 => CardRank::Jack,
            12 => CardRank::Queen,
            13 => CardRank::King,
            14 => CardRank::Ace,
            16 => return Self::SmallJoker,
            _ => return Self::BigJoker,
        };
        let suit = match card.suit {
            GuandanSuit::Diamonds => CardSuit::Diamond,
            GuandanSuit::Clubs => CardSuit::Club,
            GuandanSuit::Hearts => CardSuit::Heart,
            GuandanSuit::Spades => CardSuit::Spade,
            GuandanSuit::Joker => return Self::SmallJoker,
        };
        Self::Suited { rank, suit }
    }
}

pub(super) fn render_card_row(
    commands: &mut UiComposer,
    parent: Entity,
    cards: &[GuandanCard],
    assets: &UiAssets,
) {
    let mut sorted = cards.to_vec();
    sorted.sort_by_key(|card| (card.rank, card.suit, card.deck));
    sorted.reverse();
    let rows = spawn_node(
        commands,
        parent,
        Node {
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            row_gap: px(2),
            ..default()
        },
        None,
    );
    for chunk in sorted.chunks(8) {
        let row = spawn_node(
            commands,
            rows,
            Node {
                height: px(CardSize::FinishedHand.dimensions().1),
                ..default()
            },
            None,
        );
        for (index, card) in chunk.iter().enumerate() {
            add_card_image(
                commands,
                row,
                *card,
                CardSize::FinishedHand,
                index,
                index + 1 == chunk.len(),
                false,
                assets,
            );
        }
    }
}
