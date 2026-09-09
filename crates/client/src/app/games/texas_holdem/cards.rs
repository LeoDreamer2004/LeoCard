use crate::app::presentation::{PendingDealSound, TEXT, add_text, ease_out_cubic, spawn_node};
use crate::app::runtime::UiAssets;
use bevy::prelude::*;
use leocard_protocol::{SeatId, TABLE_SEAT_COUNT, TexasHoldemSnapshot};
use leocard_qigui523::{QiGuiRank, QiGuiSuit};
use leocard_texas_holdem::{TexasHoldemCard, TexasHoldemRank, TexasHoldemSuit};

#[derive(Component)]
pub(crate) struct TexasDealCard {
    elapsed: f32,
    delay: f32,
    offset: Vec2,
}

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
pub(crate) struct TexasBoardCardFlip {
    elapsed: f32,
    delay: f32,
    face: Handle<Image>,
    face_visible: bool,
}

pub(super) struct TexasInitialDeal {
    pub(super) duration: f32,
    pub(super) own_delays: Vec<f32>,
}

pub(super) fn add_texas_draw_pile(
    commands: &mut Commands,
    parent: Entity,
    count: usize,
    assets: &UiAssets,
) {
    let pile = spawn_node(
        commands,
        parent,
        Node {
            width: px(44),
            height: px(62),
            position_type: PositionType::Relative,
            ..default()
        },
        None,
    );
    for layer in 0..5 {
        let card = commands
            .spawn((
                Node {
                    position_type: PositionType::Absolute,
                    left: px(layer as f32 * 1.7),
                    top: px((4 - layer) as f32 * 1.0),
                    width: px(39),
                    height: px(54),
                    border: UiRect::all(px(1)),
                    border_radius: BorderRadius::all(px(4)),
                    ..default()
                },
                ImageNode::new(assets.playing_cards.card_back.clone()),
                BorderColor::all(TEXT.with_alpha(0.52)),
                ZIndex(layer),
            ))
            .id();
        commands.entity(pile).add_child(card);
    }
    let counter = spawn_node(
        commands,
        pile,
        Node {
            position_type: PositionType::Absolute,
            left: px(7),
            top: px(27),
            width: px(29),
            height: px(21),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            border_radius: BorderRadius::all(px(6)),
            ..default()
        },
        Some(Color::BLACK.with_alpha(0.68)),
    );
    commands.entity(counter).insert(ZIndex(8));
    add_text(commands, counter, count.to_string(), 12.0, TEXT, assets);
}

pub(super) fn add_texas_board_back(
    commands: &mut Commands,
    parent: Entity,
    index: usize,
    delay: Option<f32>,
    assets: &UiAssets,
) {
    let animated = delay.is_some();
    let card = commands
        .spawn((
            Node {
                width: px(56),
                height: px(76),
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
        ))
        .id();
    commands.entity(parent).add_child(card);
    commands.entity(card).insert(TexasBoardCardBack {
        elapsed: if animated { 0.0 } else { 1.0 },
        delay: delay.unwrap_or(0.0),
        start_offset: if animated {
            Vec2::new(-65.0 - index as f32 * 63.0, 0.0)
        } else {
            Vec2::ZERO
        },
        phase: index as f32 * 0.9,
    });
    if let Some(delay) = delay {
        commands.spawn(PendingDealSound {
            remaining: delay,
            variant: index % assets.audio.deal_sounds.len().max(1),
        });
    }
}

pub(super) fn add_texas_board_face(
    commands: &mut Commands,
    parent: Entity,
    card: TexasHoldemCard,
    flip_delay: Option<f32>,
    assets: &UiAssets,
) {
    let face = texas_card_face(card, assets);
    let entity = commands
        .spawn((
            Node {
                width: px(56),
                height: px(76),
                border: UiRect::all(px(1)),
                border_radius: BorderRadius::all(px(5)),
                ..default()
            },
            ImageNode::new(if flip_delay.is_some() {
                assets.playing_cards.card_back.clone()
            } else {
                face.clone()
            }),
            BorderColor::all(TEXT.with_alpha(0.42)),
            UiTransform::IDENTITY,
        ))
        .id();
    commands.entity(parent).add_child(entity);
    if let Some(delay) = flip_delay {
        commands.entity(entity).insert(TexasBoardCardFlip {
            elapsed: 0.0,
            delay,
            face,
            face_visible: false,
        });
    }
}

pub(super) fn spawn_texas_initial_deal(
    commands: &mut Commands,
    table: Entity,
    game: &TexasHoldemSnapshot,
    own_seat: SeatId,
    assets: &UiAssets,
) -> TexasInitialDeal {
    let source = Vec2::new(456.0, 258.0);
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
    funded
        .sort_by_key(|player| (player.seat.0 + TABLE_SEAT_COUNT - dealer_seat) % TABLE_SEAT_COUNT);
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
                    own_delays[own_card] = dealt as f32 * 0.07;
                    own_card += 1;
                }
                dealt += 1;
                continue;
            }
            let relative = (player.seat.0 + TABLE_SEAT_COUNT - own_seat.0) % TABLE_SEAT_COUNT;
            let target = texas_seat_card_target(relative);
            let delay = dealt as f32 * 0.07;
            let entity = commands
                .spawn((
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
                ))
                .id();
            commands.entity(table).add_child(entity);
            commands.spawn(PendingDealSound {
                remaining: delay,
                variant: dealt % assets.audio.deal_sounds.len().max(1),
            });
            dealt += 1;
        }
    }
    TexasInitialDeal {
        duration: dealt as f32 * 0.07 + 0.30,
        own_delays,
    }
}

fn texas_seat_card_target(relative: u8) -> Vec2 {
    match relative {
        0 => Vec2::new(620.0, 570.0),
        1 => Vec2::new(180.0, 400.0),
        2 => Vec2::new(180.0, 175.0),
        3 => Vec2::new(620.0, 50.0),
        4 => Vec2::new(1060.0, 175.0),
        5 => Vec2::new(1060.0, 400.0),
        _ => Vec2::new(620.0, 300.0),
    }
}

pub(super) fn add_texas_card(
    commands: &mut Commands,
    parent: Entity,
    card: TexasHoldemCard,
    size: (f32, f32),
    animation: Option<(f32, Vec2)>,
    assets: &UiAssets,
) -> Entity {
    let face = texas_card_face(card, assets);
    let entity = commands
        .spawn((
            Node {
                width: px(size.0),
                height: px(size.1),
                border_radius: BorderRadius::all(px(5)),
                ..default()
            },
            ImageNode::new(face).with_color(if animation.is_some() {
                Color::WHITE.with_alpha(0.0)
            } else {
                Color::WHITE
            }),
            UiTransform::IDENTITY,
        ))
        .id();
    commands.entity(parent).add_child(entity);
    if let Some((delay, offset)) = animation {
        commands.entity(entity).insert(TexasDealCard {
            elapsed: 0.0,
            delay,
            offset,
        });
    }
    entity
}

pub(super) fn texas_card_face(card: TexasHoldemCard, assets: &UiAssets) -> Handle<Image> {
    assets
        .playing_cards
        .cards
        .get(&(qigui_rank(card.rank()), qigui_suit(card.suit())))
        .expect("德州普通牌面应当已随公共牌组加载")
        .clone()
}

pub(super) fn animate_texas_deal_cards(
    time: Res<Time>,
    mut cards: Query<(&mut TexasDealCard, &mut UiTransform, &mut ImageNode)>,
) {
    for (mut animation, mut transform, mut image) in &mut cards {
        animation.elapsed += time.delta_secs();
        let raw = ((animation.elapsed - animation.delay) / 0.28).clamp(0.0, 1.0);
        let movement = ease_out_cubic(raw);
        transform.translation = Val2::px(
            animation.offset.x * (1.0 - movement),
            animation.offset.y * (1.0 - movement),
        );
        transform.scale = Vec2::splat(0.76 + movement * 0.24);
        image.color = Color::WHITE.with_alpha(raw);
    }
}

pub(super) fn animate_texas_flying_card_backs(
    mut commands: Commands,
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

pub(super) fn animate_texas_board_card_flips(
    time: Res<Time>,
    mut cards: Query<(&mut TexasBoardCardFlip, &mut UiTransform, &mut ImageNode)>,
) {
    for (mut animation, mut transform, mut image) in &mut cards {
        animation.elapsed += time.delta_secs();
        let progress = ((animation.elapsed - animation.delay) / 0.34).clamp(0.0, 1.0);
        if progress >= 0.5 && !animation.face_visible {
            image.image = animation.face.clone();
            animation.face_visible = true;
        }
        let horizontal = if progress < 0.5 {
            1.0 - ease_out_cubic(progress * 2.0)
        } else {
            ease_out_cubic((progress - 0.5) * 2.0)
        };
        transform.scale = Vec2::new(horizontal.max(0.025), 1.0 + (1.0 - horizontal) * 0.04);
    }
}

fn qigui_rank(rank: TexasHoldemRank) -> QiGuiRank {
    match rank {
        TexasHoldemRank::Two => QiGuiRank::Two,
        TexasHoldemRank::Three => QiGuiRank::Three,
        TexasHoldemRank::Four => QiGuiRank::Four,
        TexasHoldemRank::Five => QiGuiRank::Five,
        TexasHoldemRank::Six => QiGuiRank::Six,
        TexasHoldemRank::Seven => QiGuiRank::Seven,
        TexasHoldemRank::Eight => QiGuiRank::Eight,
        TexasHoldemRank::Nine => QiGuiRank::Nine,
        TexasHoldemRank::Ten => QiGuiRank::Ten,
        TexasHoldemRank::Jack => QiGuiRank::Jack,
        TexasHoldemRank::Queen => QiGuiRank::Queen,
        TexasHoldemRank::King => QiGuiRank::King,
        TexasHoldemRank::Ace => QiGuiRank::Ace,
    }
}

fn qigui_suit(suit: TexasHoldemSuit) -> QiGuiSuit {
    match suit {
        TexasHoldemSuit::Diamond => QiGuiSuit::Diamond,
        TexasHoldemSuit::Club => QiGuiSuit::Club,
        TexasHoldemSuit::Heart => QiGuiSuit::Heart,
        TexasHoldemSuit::Spade => QiGuiSuit::Spade,
    }
}
