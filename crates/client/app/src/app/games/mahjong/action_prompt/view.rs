use crate::app::presentation::add_text;
use crate::app::runtime::UiAssets;
use bevy::{picking::Pickable, prelude::*};
use leocard_ui::UiComposer;
use std::f32::consts;

#[derive(Component)]
pub(in super::super) struct MahjongResponseTile;

#[derive(Component)]
pub(in super::super) struct ResponseArrow {
    offset: Vec2,
    upward: Vec2,
}

pub(in super::super) fn add_mahjong_response_indicator(
    commands: &mut UiComposer,
    tile: Entity,
    size: Vec2,
    rotation: f32,
    projected: bool,
    assets: &UiAssets,
) {
    if projected {
        commands.entity(tile).insert(MahjongResponseTile);
        return;
    }
    // 抵消牌河朝向，箭头始终在屏幕上方并指向牌张。
    let inverse = Rot2::radians(-rotation);
    let upward = inverse * Vec2::NEG_Y;
    let extent = upward.x.abs() * size.x * 0.5 + upward.y.abs() * size.y * 0.5;
    let offset = upward * (extent + 24.0);
    let arrow = add_text(
        commands,
        tile,
        "▼",
        20.0,
        Color::srgb(0.95, 0.29, 0.045),
        assets,
    );
    commands.entity(arrow).insert((
        ResponseArrow { offset, upward },
        Node {
            position_type: PositionType::Absolute,
            left: percent(50),
            top: percent(50),
            width: px(32),
            height: px(24),
            margin: UiRect {
                left: px(-16),
                top: px(-12),
                ..default()
            },
            ..default()
        },
        UiTransform {
            translation: Val2::px(offset.x, offset.y),
            rotation: inverse,
            scale: Vec2::new(1.5, 1.0),
        },
        ZIndex(201),
        Pickable::IGNORE,
    ));
}

pub(in super::super) fn animate_mahjong_response_indicators(
    time: Res<Time>,
    mut arrows: Query<(&ResponseArrow, &mut UiTransform, &mut TextColor)>,
) {
    let phase = time.elapsed_secs_wrapped() * consts::TAU / 1.5;
    let breath = (phase.sin() + 1.0) * 0.5;
    for (arrow, mut transform, mut color) in &mut arrows {
        let offset = arrow.offset + arrow.upward * (breath * 6.0);
        transform.translation = Val2::px(offset.x, offset.y);
        color.0 = Color::srgb(0.95, 0.29, 0.045);
    }
}
