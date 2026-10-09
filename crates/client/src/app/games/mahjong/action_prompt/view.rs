use crate::app::presentation::{add_text, spawn_node};
use crate::app::runtime::UiAssets;
use bevy::{picking::Pickable, prelude::*};
use std::f32::consts;

const RESPONSE_GOLD: Color = Color::srgb(1.0, 0.80, 0.20);
const BREATH_DURATION: f32 = 2.4;

#[derive(Component)]
pub(in super::super) struct ResponseFrame {
    opacity: f32,
}

#[derive(Component)]
pub(in super::super) struct ResponseArrow {
    offset: Vec2,
    upward: Vec2,
}

pub(in super::super) fn add_mahjong_response_indicator(
    commands: &mut Commands,
    tile: Entity,
    size: Vec2,
    rotation: f32,
    assets: &UiAssets,
) {
    for (inset, opacity) in [(2.0, 1.0), (6.0, 0.70)] {
        let frame = spawn_node(
            commands,
            tile,
            Node {
                position_type: PositionType::Absolute,
                left: px(-inset),
                right: px(-inset),
                top: px(-inset),
                bottom: px(-inset),
                border: UiRect::all(px(1.5)),
                border_radius: BorderRadius::all(px(5.0 + inset)),
                ..default()
            },
            None,
        );
        commands.entity(frame).insert((
            ResponseFrame { opacity },
            BorderColor::all(RESPONSE_GOLD.with_alpha(opacity)),
            ZIndex(200),
            Pickable::IGNORE,
        ));
    }
    // 抵消牌河朝向，箭头始终在屏幕上方并指向牌张。
    let inverse = Rot2::radians(-rotation);
    let upward = inverse * Vec2::NEG_Y;
    let extent = upward.x.abs() * size.x * 0.5 + upward.y.abs() * size.y * 0.5;
    let offset = upward * (extent + 19.0);
    let arrow = add_text(commands, tile, "▼", 20.0, RESPONSE_GOLD, assets);
    commands.entity(arrow).insert((
        ResponseArrow { offset, upward },
        Node {
            position_type: PositionType::Absolute,
            left: percent(50),
            top: percent(50),
            width: px(24),
            height: px(24),
            margin: UiRect {
                left: px(-12),
                top: px(-12),
                ..default()
            },
            ..default()
        },
        TextLayout::justify(Justify::Center),
        UiTransform {
            translation: Val2::px(offset.x, offset.y),
            rotation: inverse,
            ..default()
        },
        ZIndex(201),
        Pickable::IGNORE,
    ));
}

pub(in super::super) fn animate_mahjong_response_indicators(
    time: Res<Time>,
    mut frames: Query<(&ResponseFrame, &mut BorderColor)>,
    mut arrows: Query<(&ResponseArrow, &mut UiTransform, &mut TextColor)>,
) {
    let phase = time.elapsed_secs_wrapped() * consts::TAU / BREATH_DURATION;
    let breath = (phase.sin() + 1.0) * 0.5;
    for (frame, mut border) in &mut frames {
        *border =
            BorderColor::all(RESPONSE_GOLD.with_alpha(frame.opacity * (0.70 + breath * 0.30)));
    }
    for (arrow, mut transform, mut color) in &mut arrows {
        let offset = arrow.offset + arrow.upward * (breath * 3.0);
        transform.translation = Val2::px(offset.x, offset.y);
        color.0 = RESPONSE_GOLD.with_alpha(0.72 + breath * 0.28);
    }
}
