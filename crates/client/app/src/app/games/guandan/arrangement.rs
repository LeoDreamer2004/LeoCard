use leocard_guandan::{GuandanCard, rank_strength};
use std::cmp::Reverse;

#[derive(Clone, Copy, Default, Eq, PartialEq)]
pub(super) enum HandLayoutMode {
    #[default]
    Columns,
    Row,
}

pub(super) struct HandColumn {
    pub cards: Vec<GuandanCard>,
}

impl HandLayoutMode {
    pub fn toggle(&mut self) {
        *self = match self {
            Self::Columns => Self::Row,
            Self::Row => Self::Columns,
        };
    }
}

pub(super) fn hand_columns(hand: &[GuandanCard], level: u8) -> Vec<HandColumn> {
    let mut columns: Vec<HandColumn> = Vec::new();
    for card in sorted_cards(hand, level) {
        if let Some(column) = columns.last_mut()
            && column.cards[0].rank == card.rank
        {
            column.cards.push(card);
        } else {
            columns.push(HandColumn { cards: vec![card] });
        }
    }
    columns
}

pub(super) fn sorted_cards(hand: &[GuandanCard], level: u8) -> Vec<GuandanCard> {
    let mut cards = hand.to_vec();
    cards.sort_by_key(|card| {
        Reverse((
            rank_strength(card.rank, level),
            card.is_wild(level),
            card.suit,
            card.deck,
        ))
    });
    cards
}
