use crate::app::presentation::{
    PlayEffectKind, SeatSide, SequenceEffectCard, TrickCardLayout, add_card_stars,
    add_sequence_play_decoration,
};
use crate::app::runtime::{CardFace, CardRank, CardSuit, UiAssets};
use bevy::picking::Pickable;
use bevy::prelude::*;
use leocard_guandan::{GuandanCard, GuandanSuit};
use leocard_ui::{UiComposer, spawn_node};
use std::cmp::Reverse;

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

pub(super) fn play_area_width(side: SeatSide) -> f32 {
    TrickCardLayout::default().row_width(cards_per_row(side))
}

fn cards_per_row(side: SeatSide) -> usize {
    if matches!(side, SeatSide::Top) { 10 } else { 6 }
}

pub(super) fn render_card_row(
    commands: &mut UiComposer,
    parent: Entity,
    cards: &[GuandanCard],
    level: u8,
    side: SeatSide,
    effect: Option<PlayEffectKind>,
    assets: &UiAssets,
) {
    let mut sorted = cards.to_vec();
    sorted.sort_by_key(|card| Reverse((card.rank, card.suit, card.deck)));
    let layout = TrickCardLayout::default();
    let (width, height, reveal) = (layout.width, layout.height, layout.reveal);
    let per_row = cards_per_row(side);
    for (row_index, chunk) in sorted.chunks(per_row).enumerate() {
        let row_width = layout.row_width(chunk.len());
        let row = spawn_node(
            commands,
            parent,
            Node {
                width: px(row_width),
                height: px(height),
                flex_shrink: 0.0,
                ..default()
            },
            None,
        );
        for (index, card) in chunk.iter().copied().enumerate() {
            let image = commands.child(
                row,
                (
                    Node {
                        position_type: PositionType::Absolute,
                        left: px(index as f32 * reveal),
                        width: px(width),
                        height: px(height),
                        border_radius: BorderRadius::all(px(4)),
                        ..default()
                    },
                    ImageNode::new(assets.playing_cards.face(card)).with_color(
                        if effect.is_some() {
                            Color::NONE
                        } else {
                            Color::WHITE
                        },
                    ),
                    ZIndex(index as i32),
                    Pickable::IGNORE,
                ),
            );
            if effect.is_some() {
                commands.entity(image).insert((
                    SequenceEffectCard {
                        index: row_index * per_row + index,
                    },
                    UiTransform::IDENTITY,
                    Visibility::Hidden,
                ));
            }
            add_level_marker(commands, image, card, level, assets);
        }
        if let Some(kind) = effect
            && (row_index + 1) * per_row >= cards.len()
        {
            add_sequence_play_decoration(commands, row, row_width, kind, assets);
        }
    }
}

pub(super) fn add_level_marker(
    commands: &mut UiComposer,
    parent: Entity,
    card: GuandanCard,
    level: u8,
    assets: &UiAssets,
) {
    if card.rank != level {
        return;
    }
    let count = if card.is_wild(level) { 2 } else { 1 };
    add_card_stars(commands, parent, count, 14.0, 4.0, assets);
}
