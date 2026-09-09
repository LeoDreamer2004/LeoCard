use super::super::{MahjongWinDecoration, MahjongWinDecorationKind, MahjongWinEffectTier};
use crate::app::presentation::spawn_node;
use bevy::prelude::*;
use bevy::ui::FocusPolicy;

fn spawn_mahjong_win_decoration(
    commands: &mut Commands,
    parent: Entity,
    node: Node,
    tier: MahjongWinEffectTier,
    reveal_duration: f32,
    kind: MahjongWinDecorationKind,
    z_index: i32,
) {
    let decoration = spawn_node(commands, parent, node, Some(Color::NONE));
    commands.entity(decoration).insert((
        MahjongWinDecoration {
            tier,
            reveal_duration,
            kind,
        },
        UiTransform::IDENTITY,
        BorderColor::all(Color::NONE),
        ZIndex(z_index),
        FocusPolicy::Pass,
    ));
}

pub(super) fn add_win_aura(
    commands: &mut Commands,
    holder: Entity,
    tier: MahjongWinEffectTier,
    reveal_duration: f32,
) {
    spawn_mahjong_win_decoration(
        commands,
        holder,
        Node {
            position_type: PositionType::Absolute,
            left: px(20),
            top: px(8),
            width: px(70),
            height: px(70),
            border_radius: BorderRadius::all(percent(50)),
            ..default()
        },
        tier,
        reveal_duration,
        MahjongWinDecorationKind::Halo,
        -3,
    );

    let ring_count = match tier {
        MahjongWinEffectTier::Normal => 1,
        MahjongWinEffectTier::HighTotal => 2,
        MahjongWinEffectTier::MajorFan => 3,
    };
    for index in 0..ring_count {
        let size = 68.0 + index as f32 * 7.0;
        spawn_mahjong_win_decoration(
            commands,
            holder,
            Node {
                position_type: PositionType::Absolute,
                left: px((110.0 - size) * 0.5),
                top: px((86.0 - size) * 0.5),
                width: px(size),
                height: px(size),
                border: UiRect::all(px(if index == 0 { 2.5 } else { 1.5 })),
                border_radius: BorderRadius::all(percent(50)),
                ..default()
            },
            tier,
            reveal_duration,
            MahjongWinDecorationKind::Ring {
                delay: index as f32 * 0.055,
                start_scale: 0.54 + index as f32 * 0.08,
                end_scale: 1.40 + index as f32 * 0.14,
                max_alpha: 0.72 - index as f32 * 0.10,
            },
            -2,
        );
    }

    let ray_count = match tier {
        MahjongWinEffectTier::Normal => 0,
        MahjongWinEffectTier::HighTotal => 6,
        MahjongWinEffectTier::MajorFan => 12,
    };
    for index in 0..ray_count {
        let angle = index as f32 / ray_count as f32 * std::f32::consts::TAU
            + std::f32::consts::FRAC_PI_4 / 2.0;
        let direction = Vec2::new(angle.cos(), angle.sin());
        let width = if tier == MahjongWinEffectTier::MajorFan && index % 3 == 0 {
            18.0
        } else {
            12.0
        };
        spawn_mahjong_win_decoration(
            commands,
            holder,
            Node {
                position_type: PositionType::Absolute,
                left: px(55.0 - width * 0.5),
                top: px(41.5),
                width: px(width),
                height: px(if index % 2 == 0 { 3.5 } else { 2.5 }),
                border_radius: BorderRadius::all(percent(50)),
                ..default()
            },
            tier,
            reveal_duration,
            MahjongWinDecorationKind::Ray {
                direction,
                distance: 42.0 + (index % 3) as f32 * 8.0,
                delay: 0.06 + (index % 4) as f32 * 0.018,
                secondary: index % 2 == 1,
            },
            -1,
        );
    }
}
