use super::state::{
    BombEffectBody, BombExplosionFlash, BombExplosionParticle, BombExplosionRing, BombFuseSpark,
    PlayEffectKind, PlayEffectState,
};
use crate::app::presentation::{ACCENT, ease_out_cubic};
use crate::app::runtime::UiAssets;
use bevy::audio::Volume;
use bevy::prelude::*;
use leocard_ui::{UiComposer, spawn_node};
use std::f32::consts;

#[expect(
    clippy::type_complexity,
    reason = "the ParamSet keeps overlapping Bevy animation queries disjoint"
)]
pub(crate) fn animate_bomb_play_effect(
    mut effect: ResMut<PlayEffectState>,
    assets: Res<UiAssets>,
    mut commands: Commands,
    mut visuals: ParamSet<(
        Query<(&BombEffectBody, &mut UiTransform, &mut Visibility)>,
        Query<(&mut UiTransform, &mut BackgroundColor), With<BombFuseSpark>>,
        Query<(&mut UiTransform, &mut BackgroundColor), With<BombExplosionFlash>>,
        Query<(&mut UiTransform, &mut BorderColor), With<BombExplosionRing>>,
        Query<(
            &BombExplosionParticle,
            &mut UiTransform,
            &mut BackgroundColor,
        )>,
    )>,
) {
    let Some(active) = effect.active.as_mut() else {
        return;
    };
    let elapsed = active.elapsed;
    let explosion_at = if matches!(active.kind, PlayEffectKind::HeavenBomb) {
        0.44
    } else {
        0.52
    };
    if matches!(
        active.kind,
        PlayEffectKind::Bomb | PlayEffectKind::HeavenBomb
    ) && elapsed >= explosion_at
        && !active.bomb_sound_played
    {
        commands.spawn((
            AudioPlayer::new(assets.audio.bomb_explosion_sound.clone()),
            PlaybackSettings {
                volume: Volume::Linear(if matches!(active.kind, PlayEffectKind::HeavenBomb) {
                    0.78
                } else {
                    0.5
                }),
                ..PlaybackSettings::DESPAWN
            },
        ));
        active.bomb_sound_played = true;
    }
    let travel = ease_out_cubic((elapsed / 0.52).clamp(0.0, 1.0));
    for (bomb, mut transform, mut visibility) in &mut visuals.p0() {
        let arc = (travel * consts::PI).sin() * 86.0;
        transform.translation = Val2::px(
            bomb.source.x * (1.0 - travel),
            bomb.source.y * (1.0 - travel) - arc,
        );
        transform.rotation = Rot2::radians(travel * 8.5);
        transform.scale = Vec2::splat(0.72 + travel * 0.36);
        *visibility = if elapsed < 0.56 {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }
    for (mut transform, mut color) in &mut visuals.p1() {
        let flicker = 0.72 + (elapsed * 72.0).sin().abs() * 0.45;
        transform.scale = Vec2::splat(flicker);
        color.0 = Color::srgba(1.0, 0.72, 0.12, if elapsed < 0.56 { 1.0 } else { 0.0 });
    }
    let explosion = ((elapsed - 0.52) / 0.58).clamp(0.0, 1.0);
    for (mut transform, mut background) in &mut visuals.p2() {
        let alpha = (1.0 - explosion).powi(2) * f32::from(elapsed >= 0.52);
        transform.scale = Vec2::splat(0.35 + explosion * 1.8);
        background.0 = Color::srgba(1.0, 0.72, 0.16, alpha * 0.82);
    }
    for (mut transform, mut border) in &mut visuals.p3() {
        transform.scale = Vec2::splat(0.28 + explosion * 2.7);
        border.set_all(ACCENT.with_alpha((1.0 - explosion) * 0.9));
    }
    for (particle, mut transform, mut background) in &mut visuals.p4() {
        let distance = particle.distance * ease_out_cubic(explosion);
        transform.translation = Val2::px(
            particle.direction.x * distance,
            particle.direction.y * distance + explosion * explosion * 34.0,
        );
        transform.rotation = Rot2::radians(explosion * 5.0 * particle.direction.x);
        transform.scale = Vec2::splat(1.0 - explosion * 0.58);
        background.0 = Color::srgba(
            1.0,
            0.28 + particle.direction.y.abs() * 0.32,
            0.05,
            (1.0 - explosion).powi(2),
        );
    }
}

pub(super) fn add_bomb_play_effect(commands: &mut UiComposer, root: Entity, source: Vec2) {
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
        let angle = index as f32 / 16.0 * consts::TAU;
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
