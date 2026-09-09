use super::super::ShengjiFailedThrowCard;
use crate::app::presentation::CardSize;
use crate::app::presentation::{HAND_CARD_REVEAL, add_text, spawn_node};
use crate::app::runtime::UiAssets;
use bevy::prelude::*;
use bevy::ui::FocusPolicy;
use leocard_protocol::{ShengjiSnapshot, ShengjiThrowFailureStage};
use leocard_qigui523::{QiGuiRank, QiGuiSuit};
use leocard_shengji::{ShengjiCard, ShengjiRank, ShengjiSuit, ShengjiTrump};

#[derive(Clone, Copy)]
pub(super) enum ShengjiCardSize {
    Hand,
    Seat,
    Score,
}

#[derive(Clone, Copy)]
struct ShengjiFailedThrowCardSpec {
    stage: ShengjiThrowFailureStage,
    direction: Vec2,
}

pub(super) fn add_shengji_card_row(
    commands: &mut Commands,
    parent: Entity,
    cards: &[ShengjiCard],
    size: ShengjiCardSize,
    trump: Option<ShengjiTrump>,
    assets: &UiAssets,
) -> Entity {
    add_shengji_card_row_internal(commands, parent, cards, size, trump, None, assets)
}

pub(super) fn add_shengji_failed_throw_card_row(
    commands: &mut Commands,
    parent: Entity,
    cards: &[ShengjiCard],
    trump: Option<ShengjiTrump>,
    stage: ShengjiThrowFailureStage,
    direction: Vec2,
    assets: &UiAssets,
) -> Entity {
    add_shengji_card_row_internal(
        commands,
        parent,
        cards,
        ShengjiCardSize::Seat,
        trump,
        Some(ShengjiFailedThrowCardSpec { stage, direction }),
        assets,
    )
}

fn add_shengji_card_row_internal(
    commands: &mut Commands,
    parent: Entity,
    cards: &[ShengjiCard],
    size: ShengjiCardSize,
    trump: Option<ShengjiTrump>,
    failed_throw: Option<ShengjiFailedThrowCardSpec>,
    assets: &UiAssets,
) -> Entity {
    let (width, height, mut reveal): (f32, f32, f32) = match size {
        ShengjiCardSize::Hand => {
            let (width, height) = CardSize::Hand.dimensions();
            (width, height, HAND_CARD_REVEAL)
        }
        ShengjiCardSize::Seat => (58.0, 79.0, 24.0),
        ShengjiCardSize::Score => (36.0, 49.0, 14.0),
    };
    if matches!(size, ShengjiCardSize::Score) && cards.len() > 1 {
        // 将两至四副牌的全部分牌压进左上角托盘，而不是裁掉尾部牌。
        reveal = reveal.min(280.0 / cards.len().saturating_sub(1) as f32);
    }
    let mut cards = cards.to_vec();
    sort_shengji_cards(&mut cards, trump);
    let last = cards.len().saturating_sub(1);
    let row = spawn_node(
        commands,
        parent,
        Node {
            height: px(height),
            flex_direction: FlexDirection::Row,
            flex_wrap: FlexWrap::NoWrap,
            align_items: AlignItems::Center,
            ..default()
        },
        None,
    );
    for (index, card) in cards.into_iter().enumerate() {
        let entity = commands
            .spawn((
                Node {
                    width: px(width),
                    height: px(height),
                    margin: UiRect::right(px(if index == last { 0.0 } else { reveal - width })),
                    border_radius: BorderRadius::all(px(4)),
                    ..default()
                },
                ImageNode::new(shengji_card_face(card, assets)),
                ZIndex(index as i32),
            ))
            .id();
        commands.entity(row).add_child(entity);
        if let Some(failed_throw) = failed_throw {
            commands.entity(entity).insert((
                ShengjiFailedThrowCard {
                    index,
                    count: last + 1,
                    stage: failed_throw.stage,
                    direction: failed_throw.direction,
                    elapsed: 0.0,
                },
                UiTransform::IDENTITY,
                Visibility::Visible,
            ));
        }
        add_shengji_trump_stars(commands, entity, card, trump, size, assets);
    }
    row
}

pub(super) fn add_shengji_trump_stars(
    commands: &mut Commands,
    card_entity: Entity,
    card: ShengjiCard,
    trump: Option<ShengjiTrump>,
    size: ShengjiCardSize,
    assets: &UiAssets,
) {
    let count = shengji_trump_star_count(card, trump);
    if count == 0 {
        return;
    }
    let (left, bottom, font_size) = match size {
        ShengjiCardSize::Hand => (4.0, 4.0, 14.0),
        ShengjiCardSize::Seat => (3.0, 3.0, 12.0),
        ShengjiCardSize::Score => (2.0, 2.0, 8.0),
    };
    let marker = add_text(
        commands,
        card_entity,
        if count == 2 { "★\n★" } else { "★" }.to_owned(),
        font_size,
        Color::srgb(1.0, 0.76, 0.08),
        assets,
    );
    commands.entity(marker).insert((
        Node {
            position_type: PositionType::Absolute,
            left: px(left),
            bottom: px(bottom),
            ..default()
        },
        TextShadow {
            offset: Vec2::new(1.0, 1.0),
            color: Color::BLACK.with_alpha(0.92),
        },
        ZIndex(5),
        FocusPolicy::Pass,
    ));
}

pub(crate) fn shengji_trump_star_count(card: ShengjiCard, trump: Option<ShengjiTrump>) -> u8 {
    let Some(trump) = trump else {
        return 0;
    };
    if !trump.is_trump(card) {
        return 0;
    }
    if matches!(card.rank(), ShengjiRank::SmallJoker | ShengjiRank::BigJoker)
        || (card.rank() == trump.level && card.suit() == trump.suit && trump.suit.is_some())
    {
        2
    } else {
        1
    }
}

pub(crate) fn sort_shengji_cards(cards: &mut [ShengjiCard], trump: Option<ShengjiTrump>) {
    cards.sort_by(|left, right| {
        shengji_display_key(*right, trump)
            .cmp(&shengji_display_key(*left, trump))
            .then_with(|| right.deck().cmp(&left.deck()))
    });
}

fn shengji_display_key(card: ShengjiCard, trump: Option<ShengjiTrump>) -> (u8, u8, u8, u8) {
    let is_trump = trump.is_some_and(|trump| trump.is_trump(card));
    let strength = trump.map_or_else(
        || shengji_rank_order(card.rank()),
        |trump| {
            if is_trump {
                trump.strength(card)
            } else {
                shengji_rank_order(card.rank())
            }
        },
    );
    let category = if is_trump {
        4
    } else {
        card.suit().map_or(4, ShengjiSuit::bid_strength)
    };
    // 所有主牌共用同一个 category；副级牌的 strength 也完全相同，因此必须
    // 在牌副编号之前再按黑、红、梅、方分组，否则两副实体牌会交错成乱序。
    let suit_order = card.suit().map_or(4, ShengjiSuit::bid_strength);
    (u8::from(is_trump), category, strength, suit_order)
}

pub(crate) fn shengji_card_face(card: ShengjiCard, assets: &UiAssets) -> Handle<Image> {
    let rank = match card.rank() {
        ShengjiRank::Two => QiGuiRank::Two,
        ShengjiRank::Three => QiGuiRank::Three,
        ShengjiRank::Four => QiGuiRank::Four,
        ShengjiRank::Five => QiGuiRank::Five,
        ShengjiRank::Six => QiGuiRank::Six,
        ShengjiRank::Seven => QiGuiRank::Seven,
        ShengjiRank::Eight => QiGuiRank::Eight,
        ShengjiRank::Nine => QiGuiRank::Nine,
        ShengjiRank::Ten => QiGuiRank::Ten,
        ShengjiRank::Jack => QiGuiRank::Jack,
        ShengjiRank::Queen => QiGuiRank::Queen,
        ShengjiRank::King => QiGuiRank::King,
        ShengjiRank::Ace => QiGuiRank::Ace,
        ShengjiRank::SmallJoker | ShengjiRank::BigJoker => QiGuiRank::Joker,
    };
    let suit = match (card.suit(), card.rank()) {
        (Some(ShengjiSuit::Diamond), _) => QiGuiSuit::Diamond,
        (Some(ShengjiSuit::Club), _) => QiGuiSuit::Club,
        (Some(ShengjiSuit::Heart), _) => QiGuiSuit::Heart,
        (Some(ShengjiSuit::Spade), _) => QiGuiSuit::Spade,
        (None, ShengjiRank::SmallJoker) => QiGuiSuit::Club,
        (None, ShengjiRank::BigJoker) => QiGuiSuit::Spade,
        _ => unreachable!("合法双升牌的花色与点数组合"),
    };
    assets
        .playing_cards
        .cards
        .get(&(rank, suit))
        .expect("双升牌面已经加载")
        .clone()
}

fn shengji_rank_order(rank: ShengjiRank) -> u8 {
    match rank {
        ShengjiRank::Two => 0,
        ShengjiRank::Three => 1,
        ShengjiRank::Four => 2,
        ShengjiRank::Five => 3,
        ShengjiRank::Six => 4,
        ShengjiRank::Seven => 5,
        ShengjiRank::Eight => 6,
        ShengjiRank::Nine => 7,
        ShengjiRank::Ten => 8,
        ShengjiRank::Jack => 9,
        ShengjiRank::Queen => 10,
        ShengjiRank::King => 11,
        ShengjiRank::Ace => 12,
        ShengjiRank::SmallJoker => 13,
        ShengjiRank::BigJoker => 14,
    }
}

pub(crate) fn shengji_current_level(game: &ShengjiSnapshot) -> ShengjiRank {
    game.trump.map_or(game.bidding_level, |trump| trump.level)
}

pub(crate) fn shengji_display_trump(game: &ShengjiSnapshot) -> Option<ShengjiTrump> {
    game.trump.or_else(|| {
        game.declaration.as_ref().map(|declaration| {
            ShengjiTrump::new(shengji_current_level(game), declaration.trump.trump_suit())
                .expect("双升级牌始终是普通点数")
                .with_constant_trump(game.rules.constant_trump)
        })
    })
}

/// 手牌排序在尚无人亮主时也使用一个仅用于排序的临时无主规则，使本局
/// 级牌紧跟在大小王右侧。渲染星标仍使用 `shengji_display_trump`，不会把
/// 尚未正式确定的主牌状态提前公开。
pub(crate) fn shengji_hand_sort_trump(game: &ShengjiSnapshot) -> Option<ShengjiTrump> {
    shengji_display_trump(game).or_else(|| {
        ShengjiTrump::new(game.bidding_level, None)
            .map(|trump| trump.with_constant_trump(game.rules.constant_trump))
            .ok()
    })
}

pub(super) fn shengji_level_label(rank: ShengjiRank) -> String {
    format!("打 {}", shengji_rank_label(rank))
}

fn shengji_rank_label(rank: ShengjiRank) -> &'static str {
    match rank {
        ShengjiRank::Two => "2",
        ShengjiRank::Three => "3",
        ShengjiRank::Four => "4",
        ShengjiRank::Five => "5",
        ShengjiRank::Six => "6",
        ShengjiRank::Seven => "7",
        ShengjiRank::Eight => "8",
        ShengjiRank::Nine => "9",
        ShengjiRank::Ten => "10",
        ShengjiRank::Jack => "J",
        ShengjiRank::Queen => "Q",
        ShengjiRank::King => "K",
        ShengjiRank::Ace => "A",
        ShengjiRank::SmallJoker => "小王",
        ShengjiRank::BigJoker => "大王",
    }
}
