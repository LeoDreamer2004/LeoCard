//! Board reveal timing and the transition between two complete runouts.
use super::{TexasShowdownSequence, TexasTableLayout, texas_board_deal_visual};
use crate::app::presentation::{
    GameSummaryAnimation, PlayingCardFlip, TableCard, TableCardMaterial, add_table_card_pile,
    ease_out_cubic, table_card_pile_top_center,
};
use crate::app::runtime::UiAssets;
use bevy::picking::Pickable;
use bevy::prelude::*;
use leocard_protocol::TexasSnapshot;
use leocard_ui::{UiComposer, spawn_node};

pub(super) const TEXAS_RUNOUT_REVEAL_DURATION: f32 = PlayingCardFlip::DEFAULT_DURATION + 1.5;

#[derive(Clone, Copy)]
pub(super) struct TexasBoardReveal {
    pub first_new: usize,
    pub batch: bool,
}

impl TexasBoardReveal {
    pub(super) fn flip_delay(self, index: usize) -> Option<f32> {
        if index < self.first_new {
            return None;
        }
        Some(if self.batch {
            0.0
        } else {
            (index - self.first_new) as f32 * 0.08
        })
    }
}

#[derive(Component)]
pub(super) struct TexasRunoutCard {
    sequence: TexasShowdownSequence,
    phase: RunoutCardPhase,
}

enum RunoutCardPhase {
    First { flip: Option<PlayingCardFlip> },
    Second { index: usize, flip: PlayingCardFlip },
}

pub(super) fn add_second_board(
    commands: &mut UiComposer,
    table: Entity,
    game: &TexasSnapshot,
    reveal: TexasBoardReveal,
    sequence: TexasShowdownSequence,
    assets: &UiAssets,
    materials: &mut Assets<TableCardMaterial>,
) {
    let layer = spawn_node(
        commands,
        table,
        Node {
            position_type: PositionType::Absolute,
            left: px(0),
            right: px(0),
            top: px(0),
            bottom: px(0),
            ..default()
        },
        None,
    );
    // Both moving rows remain above the central chip sprites (layer 35).
    commands
        .entity(layer)
        .insert((ZIndex(80), Pickable::IGNORE));
    let pile_origin = TexasTableLayout::draw_pile_center() - table_card_pile_top_center();
    let pile = spawn_node(
        commands,
        layer,
        Node {
            position_type: PositionType::Absolute,
            left: px(pile_origin.x),
            top: px(pile_origin.y),
            ..default()
        },
        None,
    );
    add_table_card_pile(
        commands,
        pile,
        usize::from(game.draw_pile_len),
        assets,
        materials,
    );
    for (run, board) in [(1, &game.community), (2, &game.second_community)] {
        for (index, card) in board.iter().copied().enumerate() {
            let origin = TexasTableLayout::board_card_origin(index);
            let face = assets.playing_cards.face(card);
            let flip = PlayingCardFlip::new(
                face.clone(),
                assets.playing_cards.card_back.clone(),
                if run == 2 {
                    TexasShowdownSequence::FLIP_START
                } else {
                    0.0
                },
            );
            let phase = if run == 2 {
                RunoutCardPhase::Second { index, flip }
            } else {
                RunoutCardPhase::First {
                    flip: (reveal.batch && index >= reveal.first_new).then_some(flip),
                }
            };
            let entity = TableCard {
                texture: if run == 2 || (reveal.batch && index >= reveal.first_new) {
                    assets.playing_cards.card_back.clone()
                } else {
                    face
                },
                size: TexasTableLayout::BOARD_CARD_SIZE,
                opacity: if run == 2 { 0.0 } else { 1.0 },
            }
            .spawn(
                commands,
                layer,
                Node {
                    position_type: PositionType::Absolute,
                    left: px(origin.x),
                    top: px(origin.y),
                    ..default()
                },
                materials,
            );
            commands
                .entity(entity)
                .init(TexasRunoutCard { sequence, phase });
        }
    }
}

pub(super) fn animate_texas_runout_cards(
    animation: Res<GameSummaryAnimation>,
    mut materials: ResMut<Assets<TableCardMaterial>>,
    mut cards: Query<(
        &TexasRunoutCard,
        &mut UiTransform,
        &MaterialNode<TableCardMaterial>,
    )>,
) {
    for (card, mut transform, handle) in &mut cards {
        let elapsed = card.sequence.board_elapsed(&animation);
        let (frame, texture, alpha) = match &card.phase {
            RunoutCardPhase::First { flip } => {
                let drop = ease_out_cubic(
                    ((elapsed - TexasShowdownSequence::DROP_START)
                        / TexasShowdownSequence::DROP_DURATION)
                        .clamp(0.0, 1.0),
                );
                let face = flip
                    .as_ref()
                    .map(|flip| flip.frame_at(card.sequence.elapsed(&animation)));
                let frame = UiTransform {
                    translation: Val2::px(0.0, drop * (TexasTableLayout::BOARD_CARD_SIZE.y + 14.0)),
                    scale: face.as_ref().map_or(Vec2::ONE, |face| face.scale),
                    ..UiTransform::IDENTITY
                };
                (frame, face.map(|face| face.texture), 1.0)
            }
            RunoutCardPhase::Second { index, flip } => {
                let origin = TexasTableLayout::board_card_origin(*index);
                let offset = TexasTableLayout::draw_pile_center()
                    - origin
                    - TexasTableLayout::BOARD_CARD_SIZE * 0.5;
                let delay = TexasShowdownSequence::DEAL_START
                    + *index as f32 * TexasShowdownSequence::DEAL_INTERVAL;
                let (mut frame, alpha) = texas_board_deal_visual(offset, elapsed - delay);
                // All five cards sample the same flip clock after the whole row lands.
                let face = flip.frame_at(elapsed);
                if elapsed >= TexasShowdownSequence::FLIP_START {
                    frame.scale = face.scale;
                }
                (frame, Some(face.texture), alpha)
            }
        };
        if *transform != frame {
            *transform = frame;
        }
        let changed = materials.get(&handle.0).is_some_and(|material| {
            texture
                .as_ref()
                .is_some_and(|texture| material.texture != *texture)
                || material.tint.alpha != alpha
        });
        if changed && let Some(mut material) = materials.get_mut(&handle.0) {
            if let Some(texture) = texture {
                material.texture = texture;
            }
            material.tint.alpha = alpha;
        }
    }
}
