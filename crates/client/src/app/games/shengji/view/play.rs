use super::super::{
    ShengjiCollectingScoreText, ShengjiFailedThrowLabel, ShengjiLevelIndicator,
    ShengjiScoreCaptureEffectState, ShengjiScoreTrayAnchor, ShengjiScoreTrayHover,
    ShengjiThrowPenaltyFloat, ShengjiThrowPenaltyScorePulse, displayed_shengji_captured_score,
};
use super::{
    ShengjiCardSize, add_shengji_card_row, add_shengji_failed_throw_card_row,
    shengji_display_trump, shengji_rank_label,
};
use crate::app::presentation::{ACCENT, DANGER, HEADER_BG, MUTED, TEXT, add_text, spawn_node};
use crate::app::runtime::UiAssets;
use crate::app::shell::SeatSide;
use bevy::picking::Pickable;
use bevy::prelude::*;
use bevy::ui::RelativeCursorPosition;
use leocard_protocol::{
    PlayerId, ShengjiPhaseView, ShengjiPublicPlay, ShengjiSnapshot, ShengjiThrowFailureStage,
};
use leocard_shengji::{ShengjiBidTrump, ShengjiCard, ShengjiRank, ShengjiSuit};

pub(super) fn add_shengji_play_area(
    commands: &mut Commands,
    parent: Entity,
    game: &ShengjiSnapshot,
    player: PlayerId,
    side: SeatSide,
    assets: &UiAssets,
    previous_trick: Option<&[ShengjiPublicPlay]>,
) {
    let area = spawn_node(
        commands,
        parent,
        Node {
            width: px(190),
            min_width: px(190),
            min_height: px(112),
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::Center,
            justify_content: match side {
                SeatSide::Left => JustifyContent::FlexStart,
                SeatSide::Top => JustifyContent::Center,
                SeatSide::Right => JustifyContent::FlexEnd,
            },
            ..default()
        },
        None,
    );
    let throw_failure = previous_trick.is_none().then_some(()).and_then(|_| {
        game.throw_failure
            .as_ref()
            .filter(|failure| failure.player == player)
    });
    let cards = if let Some(previous_trick) = previous_trick {
        previous_trick
            .iter()
            .find(|played| played.player == player)
            .map(|played| played.play.cards.as_slice())
    } else if matches!(
        game.phase,
        ShengjiPhaseView::Dealing { .. }
            | ShengjiPhaseView::BiddingGrace { .. }
            | ShengjiPhaseView::BottomCopyBurying { .. }
    ) {
        game.declaration
            .as_ref()
            .filter(|declaration| declaration.player == player)
            .map(|declaration| declaration.cards.as_slice())
    } else if let Some(failure) = throw_failure {
        Some(failure.attempted.as_slice())
    } else {
        game.trick.as_ref().and_then(|trick| {
            trick
                .plays
                .iter()
                .find(|played| played.player == player)
                .map(|played| played.play.cards.as_slice())
        })
    };
    if let Some(cards) = cards {
        if let Some(failure) = throw_failure {
            let holder = spawn_node(
                commands,
                area,
                Node {
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    row_gap: px(2),
                    ..default()
                },
                None,
            );
            commands
                .entity(holder)
                .insert((UiTransform::IDENTITY, Visibility::Visible));
            let label = add_text(commands, holder, "甩牌失败", 15.0, DANGER, assets);
            commands.entity(label).insert((
                ShengjiFailedThrowLabel {
                    returning: failure.stage == ShengjiThrowFailureStage::Returning,
                    elapsed: 0.0,
                },
                TextShadow {
                    offset: Vec2::new(1.0, 1.0),
                    color: Color::BLACK.with_alpha(0.88),
                },
            ));
            let direction = if player == game.you {
                Vec2::new(0.0, 92.0)
            } else {
                match side {
                    SeatSide::Left => Vec2::new(-88.0, 0.0),
                    SeatSide::Top => Vec2::new(0.0, -82.0),
                    SeatSide::Right => Vec2::new(88.0, 0.0),
                }
            };
            add_shengji_failed_throw_card_row(
                commands,
                holder,
                cards,
                shengji_display_trump(game),
                failure.stage,
                direction,
                assets,
            );
        } else {
            add_shengji_card_row(
                commands,
                area,
                cards,
                ShengjiCardSize::Seat,
                shengji_display_trump(game),
                assets,
            );
        }
    }
}

pub(super) fn add_shengji_own_play(
    commands: &mut Commands,
    table: Entity,
    game: &ShengjiSnapshot,
    assets: &UiAssets,
    previous_trick: Option<&[ShengjiPublicPlay]>,
) {
    let area = spawn_node(
        commands,
        table,
        Node {
            position_type: PositionType::Absolute,
            left: percent(35),
            right: percent(35),
            bottom: px(8),
            min_height: px(104),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            ..default()
        },
        None,
    );
    add_shengji_play_area(
        commands,
        area,
        game,
        game.you,
        SeatSide::Top,
        assets,
        previous_trick,
    );
}

pub(super) fn add_shengji_collecting_tray(
    commands: &mut Commands,
    table: Entity,
    game: &ShengjiSnapshot,
    cards: &[ShengjiCard],
    score_capture: &ShengjiScoreCaptureEffectState,
    assets: &UiAssets,
) {
    let tray = spawn_node(
        commands,
        table,
        Node {
            position_type: PositionType::Absolute,
            left: px(0),
            top: px(0),
            width: px(366),
            height: px(66),
            padding: UiRect::axes(px(10), px(5)),
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::Center,
            column_gap: px(8),
            border_radius: BorderRadius::all(px(8)),
            overflow: Overflow::visible(),
            ..default()
        },
        Some(Color::BLACK.with_alpha(0.30)),
    );
    commands
        .entity(tray)
        .insert((RelativeCursorPosition::default(), GlobalZIndex(30)));

    let dealer_team = game.dealer.map(|dealer| usize::from(dealer.0 % 2));
    for (label, level, color) in [
        (
            "庄家级牌",
            dealer_team.map(|team| game.levels[team]),
            ACCENT,
        ),
        (
            "闲家级牌",
            dealer_team.map(|team| game.levels[1 - team]),
            TEXT,
        ),
    ] {
        let cell = add_shengji_score_cell(commands, tray, label, 66.0, assets);
        let value = add_text(
            commands,
            cell,
            level.map_or("—", shengji_rank_label),
            28.0,
            color,
            assets,
        );
        commands
            .entity(value)
            .insert(ShengjiLevelIndicator { base_color: color });
    }

    let trump_cell = add_shengji_score_cell(commands, tray, "亮主", 120.0, assets);
    add_shengji_trump_summary(commands, trump_cell, game, assets);

    let score = add_shengji_score_cell(commands, tray, "闲家得分", 70.0, assets);
    let score_text = add_text(
        commands,
        score,
        displayed_shengji_captured_score(score_capture, shengji_tray_score(game)).to_string(),
        28.0,
        ACCENT,
        assets,
    );
    commands
        .entity(score_text)
        .insert((ShengjiScoreTrayAnchor, ShengjiCollectingScoreText));
    if game.throw_failure.as_ref().is_some_and(|failure| {
        failure.stage == ShengjiThrowFailureStage::Showing && failure.penalty_points > 0
    }) {
        commands
            .entity(score_text)
            .insert(ShengjiThrowPenaltyScorePulse { elapsed: 0.0 });
    }
    let popup = spawn_node(
        commands,
        tray,
        Node {
            position_type: PositionType::Absolute,
            left: px(0),
            top: percent(100),
            min_width: px(344),
            min_height: px(62),
            padding: UiRect::all(px(10)),
            border_radius: BorderRadius::all(px(8)),
            ..default()
        },
        Some(HEADER_BG.with_alpha(0.97)),
    );
    commands.entity(popup).insert((
        Visibility::Hidden,
        GlobalZIndex(1200),
        Pickable::IGNORE,
        BoxShadow::new(Color::BLACK.with_alpha(0.5), px(1), px(6), px(0), px(10)),
    ));
    commands
        .entity(tray)
        .insert(ShengjiScoreTrayHover { popup });
    if cards.is_empty() {
        add_text(commands, popup, "闲家尚未获得分牌", 12.0, MUTED, assets);
    } else {
        add_shengji_card_row(
            commands,
            popup,
            cards,
            ShengjiCardSize::Score,
            shengji_display_trump(game),
            assets,
        );
    }
}

fn add_shengji_score_cell(
    commands: &mut Commands,
    parent: Entity,
    label: &str,
    width: f32,
    assets: &UiAssets,
) -> Entity {
    let cell = spawn_node(
        commands,
        parent,
        Node {
            width: px(width),
            min_width: px(width),
            height: px(54),
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            row_gap: px(2),
            ..default()
        },
        None,
    );
    add_text(commands, cell, label, 11.0, MUTED, assets);
    spawn_node(
        commands,
        cell,
        Node {
            width: px(if label == "亮主" { 91 } else { 45 }),
            height: px(1),
            flex_shrink: 0.0,
            ..default()
        },
        Some(Color::srgb(0.45, 0.48, 0.49).with_alpha(0.78)),
    );
    cell
}

fn add_shengji_trump_summary(
    commands: &mut Commands,
    parent: Entity,
    game: &ShengjiSnapshot,
    assets: &UiAssets,
) {
    let row = spawn_node(
        commands,
        parent,
        Node {
            height: px(32),
            align_items: AlignItems::Center,
            column_gap: px(3),
            ..default()
        },
        None,
    );
    let declaration = game.declaration.as_ref();
    let trump = declaration
        .map(|declaration| declaration.trump)
        .or_else(|| {
            game.trump.map(|trump| {
                trump
                    .suit
                    .map_or(ShengjiBidTrump::NoTrumpSmallJoker, ShengjiBidTrump::Suit)
            })
        });
    let name = declaration
        .and_then(|declaration| {
            game.players
                .iter()
                .find(|player| player.id == declaration.player)
        })
        .or_else(|| {
            game.dealer
                .and_then(|dealer| game.players.iter().find(|player| player.id == dealer))
        })
        .map_or("待亮主", |player| player.name.as_str());
    let Some(trump) = trump else {
        add_text(commands, row, "—", 26.0, ACCENT, assets);
        return;
    };
    let count = declaration.map_or(1, |declaration| {
        declaration
            .cards
            .iter()
            .filter(|card| match declaration.trump {
                ShengjiBidTrump::Suit(suit) => {
                    card.suit() == Some(suit) && card.rank() == game.bidding_level
                }
                ShengjiBidTrump::NoTrumpSmallJoker => card.rank() == ShengjiRank::SmallJoker,
                ShengjiBidTrump::NoTrumpBigJoker => card.rank() == ShengjiRank::BigJoker,
            })
            .count()
            .max(1)
    });
    let (mark, color) = shengji_trump_mark(trump);
    let stack = spawn_node(
        commands,
        row,
        Node {
            width: px(28),
            min_width: px(28),
            height: px(30),
            flex_shrink: 0.0,
            position_type: PositionType::Relative,
            ..default()
        },
        None,
    );
    for index in 0..count {
        let top = if count == 1 {
            5.0
        } else {
            10.0 * index as f32 / (count - 1) as f32
        };
        let slot = spawn_node(
            commands,
            stack,
            Node {
                position_type: PositionType::Absolute,
                left: px(0),
                top: px(top),
                width: px(28),
                height: px(20),
                ..default()
            },
            None,
        );
        add_shengji_outlined_mark(commands, slot, mark, color, 20.0, assets);
        commands.entity(slot).insert(ZIndex(index as i32));
    }
    let name_area = spawn_node(
        commands,
        row,
        Node {
            width: px(84),
            min_width: px(84),
            flex_shrink: 0.0,
            overflow: Overflow::clip(),
            ..default()
        },
        None,
    );
    commands
        .entity(name_area)
        .insert(UiTransform::from_translation(Val2::px(0.0, 3.0)));
    let name = add_text(commands, name_area, name, 13.0, TEXT, assets);
    commands
        .entity(name)
        .insert(TextLayout::default().with_no_wrap());
}

fn shengji_trump_mark(trump: ShengjiBidTrump) -> (&'static str, Color) {
    match trump {
        ShengjiBidTrump::Suit(ShengjiSuit::Diamond) => ("♦", DANGER),
        ShengjiBidTrump::Suit(ShengjiSuit::Club) => ("♣", Color::srgb(0.79, 0.82, 0.82)),
        ShengjiBidTrump::Suit(ShengjiSuit::Heart) => ("♥", DANGER),
        ShengjiBidTrump::Suit(ShengjiSuit::Spade) => ("♠", Color::srgb(0.79, 0.82, 0.82)),
        ShengjiBidTrump::NoTrumpSmallJoker => ("NG", Color::srgb(0.65, 0.68, 0.70)),
        ShengjiBidTrump::NoTrumpBigJoker => ("NG", DANGER),
    }
}

pub(super) fn add_shengji_outlined_mark(
    commands: &mut Commands,
    parent: Entity,
    mark: &str,
    color: Color,
    size: f32,
    assets: &UiAssets,
) {
    for (dx, dy) in [(0.0, 1.0), (2.0, 1.0), (1.0, 0.0), (1.0, 2.0)] {
        let text = add_text(commands, parent, mark, size, Color::BLACK, assets);
        commands.entity(text).insert((
            Node {
                position_type: PositionType::Absolute,
                left: px(dx),
                top: px(dy),
                ..default()
            },
            Pickable::IGNORE,
        ));
    }
    let text = add_text(commands, parent, mark, size, color, assets);
    commands.entity(text).insert((
        Node {
            position_type: PositionType::Absolute,
            left: px(1),
            top: px(1),
            ..default()
        },
        Pickable::IGNORE,
    ));
}

pub(crate) fn sync_shengji_score_tray_hover(
    trays: Query<(&RelativeCursorPosition, &ShengjiScoreTrayHover)>,
    mut popups: Query<&mut Visibility>,
) {
    for (cursor, tray) in &trays {
        if let Ok(mut visibility) = popups.get_mut(tray.popup) {
            let next = if cursor.cursor_over() {
                Visibility::Visible
            } else {
                Visibility::Hidden
            };
            if *visibility != next {
                *visibility = next;
            }
        }
    }
}

pub(super) fn add_shengji_throw_penalty_effect(
    commands: &mut Commands,
    table: Entity,
    game: &ShengjiSnapshot,
    assets: &UiAssets,
) {
    let Some(failure) = game.throw_failure.as_ref().filter(|failure| {
        failure.stage == ShengjiThrowFailureStage::Showing && failure.penalty_points > 0
    }) else {
        return;
    };
    let own_seat = game
        .players
        .iter()
        .find(|player| player.id == game.you)
        .map_or(0, |player| player.seat.0);
    let offender_seat = game
        .players
        .iter()
        .find(|player| player.id == failure.player)
        .map_or(own_seat, |player| player.seat.0);
    let offender_anchor = match (offender_seat + 4 - own_seat) % 4 {
        0 => Vec2::new(50.0, 86.0),
        1 => Vec2::new(20.0, 52.0),
        2 => Vec2::new(50.0, 20.0),
        3 => Vec2::new(80.0, 52.0),
        _ => unreachable!(),
    };
    // 左上四栏得分框的最后一栏是闲家得分。
    let score_anchor = Vec2::new(26.0, 6.0);
    let dealer_side_penalty = game
        .dealer
        .is_some_and(|dealer| dealer.0 % 2 == failure.player.0 % 2);
    let (source, target, sign) = if dealer_side_penalty {
        (offender_anchor, score_anchor, "+")
    } else {
        (score_anchor, Vec2::new(-4.0, 6.0), "−")
    };
    let effect = add_text(
        commands,
        table,
        format!("{sign}{}", failure.penalty_points),
        24.0,
        Color::NONE,
        assets,
    );
    commands.entity(effect).insert((
        Node {
            position_type: PositionType::Absolute,
            left: percent(source.x),
            top: percent(source.y),
            width: px(60),
            height: px(36),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            ..default()
        },
        ShengjiThrowPenaltyFloat {
            source,
            target,
            elapsed: 0.0,
        },
        UiTransform::IDENTITY,
        TextShadow {
            offset: Vec2::new(1.0, 1.0),
            color: Color::BLACK.with_alpha(0.9),
        },
        GlobalZIndex(840),
        Pickable::IGNORE,
    ));
}

fn shengji_tray_score(game: &ShengjiSnapshot) -> u32 {
    match &game.phase {
        ShengjiPhaseView::Finished { result, .. } => i64::from(result.trick_points)
            .saturating_add(i64::from(result.penalty_adjustment))
            .max(0) as u32,
        _ => game.collecting_score,
    }
}
