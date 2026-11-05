use crate::app::presentation::{
    CardSize, PlayEffectKind, SeatSide, SequenceEffectCard, add_card_stars,
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

pub(super) const PLAY_AREA_WIDTH: f32 = 190.0;
pub(super) const CENTER_PLAY_WIDTH: f32 = 310.0;

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
    let (mut width, mut height) = CardSize::TrickSeat.dimensions();
    let center = matches!(side, SeatSide::Top);
    let area_width = if center {
        CENTER_PLAY_WIDTH
    } else {
        PLAY_AREA_WIDTH
    };
    let per_row = if center {
        if cfg!(target_os = "android") {
            // The top and bottom seats share one vertical lane on phones.
            width *= 86.0 / height;
            height = 86.0;
        }
        10
    } else if sorted.len() > 6 {
        let compact = if cfg!(target_os = "android") {
            0.64
        } else {
            0.78
        };
        width *= compact;
        height *= compact;
        5
    } else {
        if sorted.len() == 6 && cfg!(target_os = "android") {
            width *= 0.82;
            height *= 0.82;
        }
        6
    };
    for (row_index, chunk) in sorted.chunks(per_row).enumerate() {
        let gaps = chunk.len().saturating_sub(1) as f32;
        let reveal = if gaps > 0.0 {
            ((area_width - width) / gaps).min(width * 0.48)
        } else {
            width
        };
        let row_width = width + reveal * gaps;
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
