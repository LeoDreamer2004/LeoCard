use super::state::{
    PlayEffectKind, SequenceAirplane, SequenceAirplaneTrail, SequenceEffectLabel,
    SequenceEffectLabelPart, SequenceEffectMotif, SequenceFlowerPart, SequenceGuideSegment,
    SequenceWindStreak,
};
use crate::app::runtime::UiAssets;
use bevy::picking::Pickable;
use bevy::prelude::*;
use leocard_ui::{UiComposer, spawn_node};

pub(crate) fn add_sequence_play_decoration(
    commands: &mut UiComposer,
    play_area: Entity,
    cards_width: f32,
    kind: PlayEffectKind,
    assets: &UiAssets,
) {
    let Some((label, color, motif)) = kind.sequence_style() else {
        return;
    };

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
    let label_root = commands.child(
        play_area,
        (
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
        ),
    );

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
        commands.child(
            label_root,
            (
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
                Pickable::IGNORE,
            ),
        );
    }
    commands.child(
        label_root,
        (
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
            Pickable::IGNORE,
        ),
    );

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

fn add_sequence_wind(commands: &mut UiComposer, parent: Entity, cards_width: f32, color: Color) {
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
            Pickable::IGNORE,
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
            Pickable::IGNORE,
        ));
    }
}

fn add_sequence_flower(commands: &mut UiComposer, parent: Entity, cards_width: f32, color: Color) {
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
            Pickable::IGNORE,
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
        Pickable::IGNORE,
    ));
}

fn add_sequence_airplane(
    commands: &mut UiComposer,
    parent: Entity,
    cards_width: f32,
    color: Color,
    assets: &UiAssets,
) {
    commands.child(
        parent,
        (
            SequenceAirplane,
            ImageNode::new(assets.sequence_airplane.clone()).with_color(Color::NONE),
            Node {
                width: px(62.0),
                height: px(42.0),
                ..sequence_motif_node(cards_width, 29.0, -47.0)
            },
            UiTransform {
                scale: Vec2::splat(0.45),
                ..UiTransform::IDENTITY
            },
            Pickable::IGNORE,
        ),
    );

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
            Pickable::IGNORE,
        ));
    }
}
