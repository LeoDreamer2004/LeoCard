//! 七鬼五二三顺子、连对、飞机、炸弹与天炸场景表现。

use super::*;
use leocard_protocol::QiGui523Snapshot;
use leocard_protocol::{PlayerId, TABLE_SEAT_COUNT};
use leocard_qigui523::QiGuiPlayKind;

pub fn add_play_effect_overlay(
    commands: &mut Commands,
    table: Entity,
    full_screen_parent: Entity,
    game: &QiGui523Snapshot,
    effect: &ActivePlayEffect,
    assets: &UiAssets,
) {
    let parent = if matches!(effect.play.kind, QiGuiPlayKind::HeavenBomb) {
        full_screen_parent
    } else {
        table
    };
    let root = spawn_node(
        commands,
        parent,
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
    commands.entity(root).insert((
        PlayEffectRoot,
        Visibility::Hidden,
        GlobalZIndex(if matches!(effect.play.kind, QiGuiPlayKind::HeavenBomb) {
            1900
        } else {
            1100
        }),
        FocusPolicy::Pass,
    ));
    match effect.play.kind {
        QiGuiPlayKind::HeavenBomb => add_heaven_bomb_effect(commands, root, assets),
        QiGuiPlayKind::Bomb(_) => {
            add_bomb_play_effect(commands, root, bomb_effect_source(game, effect.player));
        }
        _ => {}
    }
}

pub fn add_sequence_play_decoration(
    commands: &mut Commands,
    play_area: Entity,
    card_count: usize,
    label: &str,
    color: Color,
    motif: SequenceEffectMotif,
    assets: &UiAssets,
) {
    let (width, _) = CardSize::Seat.dimensions();
    let cards_width = width + TABLE_CARD_REVEAL * card_count.saturating_sub(1) as f32;

    let guide = spawn_node(
        commands,
        play_area,
        Node {
            position_type: PositionType::Absolute,
            left: percent(50),
            // Keep the sweep line visually separate from the bottom edge of the cards.
            bottom: px(-12),
            width: px(cards_width.max(76.0)),
            height: px(12),
            margin: UiRect::left(px(-cards_width.max(76.0) * 0.5)),
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::Center,
            column_gap: px(2),
            ..default()
        },
        None,
    );
    const LINE_SEGMENTS: usize = 9;
    for index in 0..LINE_SEGMENTS {
        let segment = spawn_node(
            commands,
            guide,
            Node {
                height: px(if index % 3 == 1 { 4 } else { 3 }),
                flex_grow: if index == 0 || index == LINE_SEGMENTS - 1 {
                    0.55
                } else {
                    1.0
                },
                border_radius: BorderRadius::all(px(3)),
                ..default()
            },
            Some(color.with_alpha(0.0)),
        );
        commands.entity(segment).insert((
            SequenceGuideSegment {
                index,
                count: LINE_SEGMENTS,
            },
            UiTransform {
                rotation: Rot2::radians(match index % 4 {
                    0 => -0.045,
                    1 => 0.025,
                    2 => -0.015,
                    _ => 0.04,
                }),
                scale: Vec2::new(0.0, 1.0),
                ..UiTransform::IDENTITY
            },
        ));
    }
    let mut font = TextFont::from_font_size(27.0).with_font(assets.font.clone());
    font.style = FontStyle::Oblique(Some(14.0));
    font.weight = FontWeight::BOLD;
    font.width = FontWidth::SEMI_EXPANDED;
    // Animate one container and keep all glyph layers inside it.  This makes the
    // outline move and fade as a single label instead of letting several copied
    // text nodes drift independently.
    let label_root = commands
        .spawn((
            SequenceEffectLabel,
            Node {
                position_type: PositionType::Absolute,
                left: percent(50),
                bottom: px(-5),
                // Give the text block real horizontal space. Without this, the
                // absolutely-positioned children inherit a zero-width bound and
                // CJK text wraps after every character.
                width: px(84),
                height: px(40),
                margin: UiRect::left(px(cards_width * 0.5 + 9.0)),
                ..default()
            },
            UiTransform::IDENTITY,
        ))
        .id();
    commands.entity(play_area).add_child(label_root);

    // Bevy UI text has a single drop shadow but no native glyph stroke.  Eight
    // tightly packed copies form a stable, even outline; they share the animated
    // parent above, so only their opacity changes per frame.
    const OUTLINE_OFFSETS: [Vec2; 8] = [
        Vec2::new(-0.42, -0.42),
        Vec2::new(0.0, -0.6),
        Vec2::new(0.42, -0.42),
        Vec2::new(0.6, 0.0),
        Vec2::new(0.42, 0.42),
        Vec2::new(0.0, 0.6),
        Vec2::new(-0.42, 0.42),
        Vec2::new(-0.6, 0.0),
    ];
    for offset in OUTLINE_OFFSETS {
        let outline = commands
            .spawn((
                Text::new(label),
                font.clone(),
                TextLayout::no_wrap(),
                TextColor(Color::NONE),
                SequenceEffectLabelPart { outline: true },
                Node {
                    position_type: PositionType::Absolute,
                    left: px(offset.x),
                    top: px(offset.y),
                    ..default()
                },
                FocusPolicy::Pass,
            ))
            .id();
        commands.entity(label_root).add_child(outline);
    }
    let fill = commands
        .spawn((
            Text::new(label),
            font,
            TextLayout::no_wrap(),
            TextColor(Color::NONE),
            SequenceEffectLabelPart { outline: false },
            Node {
                position_type: PositionType::Absolute,
                left: px(0),
                top: px(0),
                ..default()
            },
            FocusPolicy::Pass,
        ))
        .id();
    commands.entity(label_root).add_child(fill);

    match motif {
        SequenceEffectMotif::Wind => add_sequence_wind(commands, play_area, cards_width, color),
        SequenceEffectMotif::Flower => add_sequence_flower(commands, play_area, cards_width, color),
        SequenceEffectMotif::Airplane => {
            add_sequence_airplane(commands, play_area, cards_width, color, assets)
        }
    }
}

fn sequence_motif_node(cards_width: f32, offset_x: f32, offset_y: f32) -> Node {
    Node {
        position_type: PositionType::Absolute,
        left: percent(50),
        bottom: px(offset_y),
        margin: UiRect::left(px(cards_width * 0.5 + offset_x)),
        ..default()
    }
}

fn add_sequence_wind(commands: &mut Commands, parent: Entity, cards_width: f32, color: Color) {
    for index in 0..5 {
        let width = [48.0, 30.0, 58.0, 37.0, 24.0][index];
        let bottom = -22.0 - (index % 3) as f32 * 5.0;
        let streak = spawn_node(
            commands,
            parent,
            Node {
                width: px(width),
                height: px(if index % 2 == 0 { 3.0 } else { 2.0 }),
                border_radius: BorderRadius::all(px(4)),
                ..sequence_motif_node(cards_width, 8.0, bottom)
            },
            Some(color.with_alpha(0.0)),
        );
        commands.entity(streak).insert((
            SequenceWindStreak { index },
            UiTransform::IDENTITY,
            FocusPolicy::Pass,
        ));
    }

    // The small detached gusts keep the wind from looking like another underline.
    for index in 5..8 {
        let streak = spawn_node(
            commands,
            parent,
            Node {
                width: px(9.0 + (index - 5) as f32 * 3.0),
                height: px(2.0),
                border_radius: BorderRadius::all(px(3)),
                ..sequence_motif_node(cards_width, 8.0, -25.0)
            },
            Some(color.with_alpha(0.0)),
        );
        commands.entity(streak).insert((
            SequenceWindStreak { index },
            UiTransform::IDENTITY,
            FocusPolicy::Pass,
        ));
    }
}

fn add_sequence_flower(commands: &mut Commands, parent: Entity, cards_width: f32, color: Color) {
    const PETAL_COUNT: usize = 7;
    for index in 0..PETAL_COUNT {
        let petal = spawn_node(
            commands,
            parent,
            Node {
                width: px(9.0),
                height: px(18.0),
                border_radius: BorderRadius::all(percent(50)),
                ..sequence_motif_node(cards_width, 80.0, -22.0)
            },
            Some(color.with_alpha(0.0)),
        );
        commands.entity(petal).insert((
            SequenceFlowerPart { index, petal: true },
            UiTransform {
                scale: Vec2::ZERO,
                ..UiTransform::IDENTITY
            },
            FocusPolicy::Pass,
        ));
    }
    let center = spawn_node(
        commands,
        parent,
        Node {
            width: px(11.0),
            height: px(11.0),
            border_radius: BorderRadius::all(percent(50)),
            ..sequence_motif_node(cards_width, 79.0, -18.0)
        },
        Some(color.with_alpha(0.0)),
    );
    commands.entity(center).insert((
        SequenceFlowerPart {
            index: PETAL_COUNT,
            petal: false,
        },
        UiTransform {
            scale: Vec2::ZERO,
            ..UiTransform::IDENTITY
        },
        FocusPolicy::Pass,
    ));
}

fn add_sequence_airplane(
    commands: &mut Commands,
    parent: Entity,
    cards_width: f32,
    color: Color,
    assets: &UiAssets,
) {
    let airplane = commands
        .spawn((
            SequenceAirplane,
            ImageNode::new(assets.games.sequence_airplane.clone()).with_color(Color::NONE),
            Node {
                width: px(62.0),
                height: px(42.0),
                ..sequence_motif_node(cards_width, 29.0, -47.0)
            },
            UiTransform {
                scale: Vec2::splat(0.45),
                ..UiTransform::IDENTITY
            },
            FocusPolicy::Pass,
        ))
        .id();
    commands.entity(parent).add_child(airplane);

    for index in 0..4 {
        let trail = spawn_node(
            commands,
            parent,
            Node {
                width: px(16.0 - index as f32 * 2.0),
                height: px(4.0),
                border_radius: BorderRadius::all(percent(50)),
                ..sequence_motif_node(cards_width, 27.0, -28.0)
            },
            Some(color.with_alpha(0.0)),
        );
        commands.entity(trail).insert((
            SequenceAirplaneTrail { index },
            UiTransform::IDENTITY,
            FocusPolicy::Pass,
        ));
    }
}

fn bomb_effect_source(game: &QiGui523Snapshot, player: PlayerId) -> Vec2 {
    let own_seat = game
        .players
        .iter()
        .find(|state| state.id == game.you)
        .map_or(0, |state| state.seat.0);
    let player_seat = game
        .players
        .iter()
        .find(|state| state.id == player)
        .map_or(own_seat, |state| state.seat.0);
    match (player_seat + TABLE_SEAT_COUNT - own_seat) % TABLE_SEAT_COUNT {
        0 => Vec2::new(0.0, 225.0),
        1 => Vec2::new(-385.0, 155.0),
        2 => Vec2::new(-385.0, -110.0),
        3 => Vec2::new(0.0, -145.0),
        4 => Vec2::new(385.0, -110.0),
        _ => Vec2::new(385.0, 155.0),
    }
}

fn add_bomb_play_effect(commands: &mut Commands, root: Entity, source: Vec2) {
    let center_node = |size: f32| Node {
        position_type: PositionType::Absolute,
        left: percent(50),
        top: percent(45),
        width: px(size),
        height: px(size),
        margin: UiRect::new(px(-size * 0.5), px(0), px(-size * 0.5), px(0)),
        ..default()
    };

    let flash = spawn_node(
        commands,
        root,
        Node {
            border_radius: BorderRadius::all(percent(50)),
            ..center_node(132.0)
        },
        Some(Color::NONE),
    );
    commands
        .entity(flash)
        .insert((BombExplosionFlash, UiTransform::IDENTITY));
    let ring = spawn_node(
        commands,
        root,
        Node {
            border: UiRect::all(px(4)),
            border_radius: BorderRadius::all(percent(50)),
            ..center_node(86.0)
        },
        None,
    );
    commands.entity(ring).insert((
        BombExplosionRing,
        BorderColor::all(Color::NONE),
        UiTransform::IDENTITY,
    ));

    for index in 0..16 {
        let angle = index as f32 / 16.0 * std::f32::consts::TAU;
        let direction = Vec2::new(angle.cos(), angle.sin());
        let particle = spawn_node(
            commands,
            root,
            Node {
                position_type: PositionType::Absolute,
                left: percent(50),
                top: percent(45),
                width: px(if index % 3 == 0 { 12 } else { 8 }),
                height: px(if index % 2 == 0 { 5 } else { 8 }),
                margin: UiRect::new(px(-4), px(0), px(-4), px(0)),
                border_radius: BorderRadius::all(px(3)),
                ..default()
            },
            Some(Color::NONE),
        );
        commands.entity(particle).insert((
            BombExplosionParticle {
                direction,
                distance: 92.0 + (index % 4) as f32 * 18.0,
            },
            UiTransform::IDENTITY,
        ));
    }

    let bomb = spawn_node(
        commands,
        root,
        Node {
            border: UiRect::all(px(3)),
            border_radius: BorderRadius::all(percent(50)),
            ..center_node(58.0)
        },
        Some(Color::srgb(0.055, 0.065, 0.07)),
    );
    commands.entity(bomb).insert((
        BombEffectBody { source },
        BorderColor::all(Color::srgb(0.28, 0.31, 0.30)),
        UiTransform::IDENTITY,
        BoxShadow::new(Color::BLACK.with_alpha(0.62), px(3), px(7), px(0), px(8)),
    ));
    let highlight = spawn_node(
        commands,
        bomb,
        Node {
            position_type: PositionType::Absolute,
            left: px(12),
            top: px(10),
            width: px(13),
            height: px(8),
            border_radius: BorderRadius::all(percent(50)),
            ..default()
        },
        Some(Color::WHITE.with_alpha(0.20)),
    );
    commands.entity(highlight).insert(UiTransform {
        rotation: Rot2::radians(-0.48),
        ..UiTransform::IDENTITY
    });
    let fuse = spawn_node(
        commands,
        bomb,
        Node {
            position_type: PositionType::Absolute,
            left: px(39),
            top: px(-7),
            width: px(25),
            height: px(5),
            border_radius: BorderRadius::all(px(3)),
            ..default()
        },
        Some(Color::srgb(0.58, 0.38, 0.18)),
    );
    commands.entity(fuse).insert(UiTransform {
        rotation: Rot2::radians(-0.55),
        ..UiTransform::IDENTITY
    });
    let spark = spawn_node(
        commands,
        bomb,
        Node {
            position_type: PositionType::Absolute,
            left: px(60),
            top: px(-15),
            width: px(13),
            height: px(13),
            border_radius: BorderRadius::all(percent(50)),
            ..default()
        },
        Some(Color::NONE),
    );
    commands
        .entity(spark)
        .insert((BombFuseSpark, UiTransform::IDENTITY));
}

fn add_heaven_bomb_effect(commands: &mut Commands, root: Entity, assets: &UiAssets) {
    let backdrop = spawn_node(
        commands,
        root,
        Node {
            position_type: PositionType::Absolute,
            left: px(0),
            right: px(0),
            top: px(0),
            bottom: px(0),
            ..default()
        },
        Some(Color::NONE),
    );
    commands.entity(backdrop).insert(HeavenBombBackdrop);

    let center_node = |size: f32| Node {
        position_type: PositionType::Absolute,
        left: percent(50),
        top: percent(45),
        width: px(size),
        height: px(size),
        margin: UiRect::new(px(-size * 0.5), px(0), px(-size * 0.5), px(0)),
        ..default()
    };

    let flash = spawn_node(
        commands,
        root,
        Node {
            border_radius: BorderRadius::all(percent(50)),
            ..center_node(250.0)
        },
        Some(Color::NONE),
    );
    commands
        .entity(flash)
        .insert((HeavenBombFlash, UiTransform::IDENTITY));

    for index in 0..14 {
        let width = 880.0 + (index % 3) as f32 * 95.0;
        let ray = spawn_node(
            commands,
            root,
            Node {
                position_type: PositionType::Absolute,
                left: percent(50),
                top: percent(45),
                width: px(width),
                height: px(if index % 4 == 0 { 7 } else { 3 }),
                margin: UiRect::new(px(-width * 0.5), px(0), px(-2), px(0)),
                border_radius: BorderRadius::all(px(4)),
                ..default()
            },
            Some(Color::NONE),
        );
        commands.entity(ray).insert((
            HeavenBombRay { index },
            UiTransform {
                rotation: Rot2::radians(
                    index as f32 / 14.0 * std::f32::consts::PI + 0.035 * (index % 2) as f32,
                ),
                scale: Vec2::new(0.0, 1.0),
                ..UiTransform::IDENTITY
            },
        ));
    }

    for (index, delay) in [0.0, 0.13, 0.27].into_iter().enumerate() {
        let ring = spawn_node(
            commands,
            root,
            Node {
                border: UiRect::all(px(if index == 0 { 7 } else { 3 })),
                border_radius: BorderRadius::all(percent(50)),
                ..center_node(112.0)
            },
            None,
        );
        commands.entity(ring).insert((
            HeavenBombShockRing { delay },
            BorderColor::all(Color::NONE),
            UiTransform::IDENTITY,
        ));
    }

    for index in 0..36 {
        let angle = index as f32 / 36.0 * std::f32::consts::TAU + (index % 5) as f32 * 0.031;
        let direction = Vec2::new(angle.cos(), angle.sin());
        let particle = spawn_node(
            commands,
            root,
            Node {
                position_type: PositionType::Absolute,
                left: percent(50),
                top: percent(45),
                width: px(if index % 6 == 0 { 18 } else { 8 }),
                height: px(if index % 4 == 0 { 5 } else { 9 }),
                margin: UiRect::new(px(-4), px(0), px(-4), px(0)),
                border_radius: BorderRadius::all(px(4)),
                ..default()
            },
            Some(Color::NONE),
        );
        commands.entity(particle).insert((
            HeavenBombParticle {
                direction,
                distance: 260.0 + (index % 7) as f32 * 45.0,
                delay: (index % 6) as f32 * 0.018,
            },
            UiTransform::IDENTITY,
        ));
    }

    let title = spawn_node(
        commands,
        root,
        Node {
            position_type: PositionType::Absolute,
            left: percent(50),
            top: percent(45),
            width: px(430),
            height: px(112),
            margin: UiRect::new(px(-215), px(0), px(-56), px(0)),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            border: UiRect::axes(px(3), px(1)),
            border_radius: BorderRadius::all(px(16)),
            ..default()
        },
        Some(Color::NONE),
    );
    commands.entity(title).insert((
        HeavenBombTitle,
        BorderColor::all(Color::NONE),
        UiTransform::IDENTITY,
        BoxShadow::new(Color::NONE, px(0), px(0), px(0), px(0)),
    ));
    let text = add_text(commands, title, "天  炸", 58.0, Color::NONE, assets);
    commands.entity(text).insert((
        HeavenBombTitleText,
        TextShadow {
            offset: Vec2::new(2.0, 4.0),
            color: Color::BLACK.with_alpha(0.88),
        },
    ));
}
