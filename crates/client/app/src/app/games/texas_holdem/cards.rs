use super::TexasTableLayout;
use crate::app::presentation::{
    CARD_DEAL_INTERVAL, PendingDealSound, PlayingCardFlip, TEXT, TablePieceDeal, ease_out_cubic,
};
use crate::app::runtime::UiAssets;
use bevy::picking::Pickable;
use bevy::prelude::*;
use leocard_protocol::TexasHoldemSnapshot;
use leocard_texas_holdem::{TexasHoldemCard, TexasHoldemRuleSet};
use leocard_ui::UiComposer;

#[derive(Component)]
pub(crate) struct TexasFlyingCardBack {
    elapsed: f32,
    delay: f32,
    source: Vec2,
    target: Vec2,
}

#[derive(Component)]
pub(crate) struct TexasBoardCardBack {
    elapsed: f32,
    delay: f32,
    start_offset: Vec2,
    phase: f32,
}

#[derive(Component)]
pub(crate) struct TexasBoardCard;

pub(super) struct TexasInitialDeal {
    pub(super) duration: f32,
    pub(super) own_delays: Vec<f32>,
    pub(super) source: Entity,
}

pub(super) fn add_texas_board_back(
    commands: &mut UiComposer,
    parent: Entity,
    index: usize,
    delay: Option<f32>,
    assets: &UiAssets,
) {
    let animated = delay.is_some();
    let card = commands.child(
        parent,
        (
            Node {
                width: px(TexasTableLayout::BOARD_CARD_SIZE.x),
                height: px(TexasTableLayout::BOARD_CARD_SIZE.y),
                flex_shrink: 0.0,
                border: UiRect::all(px(1)),
                border_radius: BorderRadius::all(px(5)),
                ..default()
            },
            ImageNode::new(assets.playing_cards.card_back.clone()).with_color(if animated {
                Color::WHITE.with_alpha(0.0)
            } else {
                Color::WHITE
            }),
            UiTransform::IDENTITY,
        ),
    );
    commands.entity(card).init(TexasBoardCardBack {
        elapsed: if animated { 0.0 } else { 1.0 },
        delay: delay.unwrap_or(0.0),
        start_offset: if animated {
            let x = if cfg!(target_os = "android") {
                TexasTableLayout::draw_pile_center().x
                    - TexasTableLayout::board_card_origin(index).x
                    - TexasTableLayout::BOARD_CARD_SIZE.x * 0.5
            } else {
                -65.0 - index as f32 * 63.0
            };
            let y = if cfg!(target_os = "android") {
                TexasTableLayout::draw_pile_center().y
                    - TexasTableLayout::board_card_origin(index).y
                    - TexasTableLayout::BOARD_CARD_SIZE.y * 0.5
            } else {
                0.0
            };
            Vec2::new(x, y)
        } else {
            Vec2::ZERO
        },
        phase: index as f32 * 0.9,
    });
    if let Some(delay) = delay {
        commands.effect(PendingDealSound {
            remaining: delay,
            variant: index % assets.audio.deal_sounds.len().max(1),
        });
    }
}

pub(super) fn add_texas_board_face(
    commands: &mut UiComposer,
    parent: Entity,
    card: TexasHoldemCard,
    flip_delay: Option<f32>,
    assets: &UiAssets,
) {
    let face = assets.playing_cards.face(card);
    let entity = commands.child(
        parent,
        (
            Node {
                width: px(TexasTableLayout::BOARD_CARD_SIZE.x),
                height: px(TexasTableLayout::BOARD_CARD_SIZE.y),
                flex_shrink: 0.0,
                border: UiRect::all(px(1)),
                border_radius: BorderRadius::all(px(5)),
                ..default()
            },
            ImageNode::new(face.clone()),
            BorderColor::all(TEXT.with_alpha(0.42)),
            UiTransform::IDENTITY,
        ),
    );
    commands.entity(entity).insert(TexasBoardCard);
    if let Some(delay) = flip_delay {
        commands.entity(entity).init(PlayingCardFlip::new(
            face,
            assets.playing_cards.card_back.clone(),
            delay,
        ));
    }
}

pub(super) fn spawn_texas_initial_deal(
    commands: &mut UiComposer,
    table: Entity,
    game: &TexasHoldemSnapshot,
    assets: &UiAssets,
) -> TexasInitialDeal {
    let source = TexasTableLayout::initial_deal_source();
    let source_anchor = commands.effect_child(
        table,
        (
            Node {
                position_type: PositionType::Absolute,
                left: px(source.x + 18.0),
                top: px(source.y + 24.5),
                width: px(0),
                height: px(0),
                ..default()
            },
            Pickable::IGNORE,
        ),
    );
    let dealer_seat = game
        .players
        .iter()
        .find(|player| player.id == game.dealer)
        .map_or(0, |player| player.seat.0);
    let mut funded = game
        .players
        .iter()
        .filter(|player| !player.folded)
        .collect::<Vec<_>>();
    funded.sort_by_key(|player| {
        (player.seat.0 + TexasHoldemRuleSet::MAX_PLAYERS - dealer_seat)
            % TexasHoldemRuleSet::MAX_PLAYERS
    });
    if !funded.is_empty() {
        funded.rotate_left(1);
    }
    let mut dealt = 0usize;
    let card_count = game.your_hole_cards.len();
    let mut own_delays = vec![0.0; card_count];
    let mut own_card = 0usize;
    for _ in 0..card_count {
        for player in &funded {
            if player.id == game.you {
                if own_card < own_delays.len() {
                    own_delays[own_card] = dealt as f32 * CARD_DEAL_INTERVAL;
                    own_card += 1;
                }
                dealt += 1;
                continue;
            }
            let layout = TexasTableLayout::for_game(game);
            let relative = layout.player_slot(game, player.id);
            let target = texas_seat_card_target(layout, relative);
            let delay = dealt as f32 * CARD_DEAL_INTERVAL;
            commands.effect_child(
                table,
                (
                    Node {
                        position_type: PositionType::Absolute,
                        left: px(source.x),
                        top: px(source.y),
                        width: px(36),
                        height: px(49),
                        border_radius: BorderRadius::all(px(3)),
                        ..default()
                    },
                    ImageNode::new(assets.playing_cards.card_back.clone())
                        .with_color(Color::WHITE.with_alpha(0.0)),
                    UiTransform::IDENTITY,
                    ZIndex(900 + dealt as i32),
                    TexasFlyingCardBack {
                        elapsed: 0.0,
                        delay,
                        source,
                        target,
                    },
                ),
            );
            commands.effect(PendingDealSound {
                remaining: delay,
                variant: dealt % assets.audio.deal_sounds.len().max(1),
            });
            dealt += 1;
        }
    }
    TexasInitialDeal {
        duration: dealt as f32 * CARD_DEAL_INTERVAL + 0.30,
        own_delays,
        source: source_anchor,
    }
}

fn texas_seat_card_target(layout: TexasTableLayout, relative: u8) -> Vec2 {
    let zone = layout.chip_zone(relative);
    Vec2::new(
        zone.left + zone.width * 0.5 - 18.0,
        zone.top + zone.height - 53.0,
    )
}

pub(super) fn add_texas_card(
    commands: &mut UiComposer,
    parent: Entity,
    card: TexasHoldemCard,
    size: (f32, f32),
    animation: Option<TablePieceDeal>,
    assets: &UiAssets,
) -> Entity {
    let face = assets.playing_cards.face(card);
    let entity = commands.child(
        parent,
        (
            Node {
                width: px(size.0),
                height: px(size.1),
                border_radius: BorderRadius::all(px(5)),
                ..default()
            },
            ImageNode::new(face),
            UiTransform::IDENTITY,
        ),
    );
    if let Some(animation) = animation {
        commands.entity(entity).init(animation);
    }
    entity
}

pub(super) fn animate_texas_flying_card_backs(
    mut commands: UiComposer,
    time: Res<Time>,
    mut cards: Query<(
        Entity,
        &mut TexasFlyingCardBack,
        &mut Node,
        &mut UiTransform,
        &mut ImageNode,
    )>,
) {
    for (entity, mut animation, mut node, mut transform, mut image) in &mut cards {
        animation.elapsed += time.delta_secs();
        let raw = ((animation.elapsed - animation.delay) / 0.28).clamp(0.0, 1.0);
        if animation.elapsed < animation.delay {
            image.color = Color::WHITE.with_alpha(0.0);
            continue;
        }
        let movement = ease_out_cubic(raw);
        let position = animation.source.lerp(animation.target, movement);
        node.left = px(position.x);
        node.top = px(position.y);
        transform.rotation = Rot2::radians((1.0 - movement) * 0.08);
        transform.scale = Vec2::splat(0.88 + movement * 0.12);
        image.color = Color::WHITE.with_alpha((raw * 5.0).min(1.0));
        if raw >= 1.0 {
            commands.entity(entity).despawn();
        }
    }
}

pub(super) fn animate_texas_board_card_backs(
    time: Res<Time>,
    mut cards: Query<(&mut TexasBoardCardBack, &mut UiTransform, &mut ImageNode)>,
) {
    for (mut animation, mut transform, mut image) in &mut cards {
        animation.elapsed += time.delta_secs();
        let raw = ((animation.elapsed - animation.delay) / 0.25).clamp(0.0, 1.0);
        if animation.elapsed < animation.delay {
            image.color = Color::WHITE.with_alpha(0.0);
            transform.translation = Val2::px(animation.start_offset.x, animation.start_offset.y);
            continue;
        }
        let movement = ease_out_cubic(raw);
        let idle = (time.elapsed_secs() * 1.7 + animation.phase).sin();
        transform.translation = Val2::px(
            animation.start_offset.x * (1.0 - movement),
            animation.start_offset.y * (1.0 - movement) + idle * 0.8,
        );
        transform.rotation = Rot2::radians(idle * 0.006);
        transform.scale = Vec2::splat(0.90 + movement * 0.10);
        image.color = Color::WHITE.with_alpha(raw);
    }
}
