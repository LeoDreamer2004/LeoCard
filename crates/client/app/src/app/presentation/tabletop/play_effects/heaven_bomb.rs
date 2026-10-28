use super::state::{
    HeavenBombBackdrop, HeavenBombFlash, HeavenBombParticle, HeavenBombRay, HeavenBombShockRing,
    HeavenBombTitle, HeavenBombTitleText, PlayEffectKind, PlayEffectState,
};
use crate::app::presentation::{add_text, ease_out_cubic};
use crate::app::runtime::UiAssets;
use bevy::prelude::*;
use leocard_ui::{UiComposer, spawn_node};
use std::f32::consts;

#[expect(
    clippy::type_complexity,
    reason = "the ParamSet keeps overlapping Bevy animation queries disjoint"
)]
pub(crate) fn animate_heaven_bomb_play_effect(
    effect: Res<PlayEffectState>,
    mut visuals: ParamSet<(
        Query<&mut BackgroundColor, With<HeavenBombBackdrop>>,
        Query<(&mut UiTransform, &mut BackgroundColor), With<HeavenBombFlash>>,
        Query<(&HeavenBombRay, &mut UiTransform, &mut BackgroundColor)>,
        Query<(&HeavenBombShockRing, &mut UiTransform, &mut BorderColor)>,
        Query<(&HeavenBombParticle, &mut UiTransform, &mut BackgroundColor)>,
        Query<
            (
                &mut UiTransform,
                &mut BackgroundColor,
                &mut BorderColor,
                &mut BoxShadow,
            ),
            With<HeavenBombTitle>,
        >,
        Query<&mut TextColor, With<HeavenBombTitleText>>,
    )>,
) {
    let Some(active) = effect
        .active
        .as_ref()
        .filter(|active| matches!(active.kind, PlayEffectKind::HeavenBomb))
    else {
        return;
    };
    let elapsed = active.elapsed;
    let life_fade = ((2.25 - elapsed) / 0.52).clamp(0.0, 1.0);
    let darken = ease_out_cubic((elapsed / 0.24).clamp(0.0, 1.0));
    for mut background in &mut visuals.p0() {
        background.0 = Color::srgba(0.008, 0.014, 0.012, darken * life_fade * 0.76);
    }

    let blast = ((elapsed - 0.40) / 0.40).clamp(0.0, 1.0);
    for (mut transform, mut background) in &mut visuals.p1() {
        let flash_alpha = (1.0 - blast).powi(3) * f32::from(elapsed >= 0.40);
        transform.scale = Vec2::splat(0.18 + ease_out_cubic(blast) * 5.8);
        transform.rotation = Rot2::radians(blast * 0.34);
        background.0 = Color::srgba(1.0, 0.78, 0.22, flash_alpha * 0.96);
    }

    for (ray, mut transform, mut background) in &mut visuals.p2() {
        let local = elapsed - 0.10 - ray.index as f32 * 0.008;
        let reveal = ease_out_cubic((local / 0.34).clamp(0.0, 1.0));
        let fade = ((1.55 - local) / 0.72).clamp(0.0, 1.0) * life_fade;
        let pulse = 0.76 + (elapsed * 15.0 + ray.index as f32 * 0.8).sin().abs() * 0.34;
        transform.scale = Vec2::new(reveal * (0.92 + blast * 0.22), 0.5 + reveal * 0.8);
        background.0 = if ray.index % 3 == 0 {
            Color::srgba(1.0, 0.86, 0.35, reveal * fade * pulse * 0.72)
        } else {
            Color::srgba(1.0, 0.56, 0.08, reveal * fade * pulse * 0.46)
        };
    }

    for (ring, mut transform, mut border) in &mut visuals.p3() {
        let local = elapsed - 0.38 - ring.delay;
        let progress = (local / 0.88).clamp(0.0, 1.0);
        let alpha = (1.0 - progress).powf(1.5) * f32::from(local >= 0.0) * life_fade;
        transform.scale = Vec2::splat(0.18 + ease_out_cubic(progress) * 6.4);
        transform.rotation = Rot2::radians(progress * (0.18 + ring.delay));
        border.set_all(Color::srgba(1.0, 0.78, 0.22, alpha * 0.95));
    }

    for (particle, mut transform, mut background) in &mut visuals.p4() {
        let local = elapsed - 0.42 - particle.delay;
        let progress = (local / 1.12).clamp(0.0, 1.0);
        let distance = particle.distance * ease_out_cubic(progress);
        transform.translation = Val2::px(
            particle.direction.x * distance,
            particle.direction.y * distance + progress * progress * 38.0,
        );
        transform.rotation = Rot2::radians(progress * 4.8 * particle.direction.x);
        transform.scale = Vec2::splat((1.25 - progress * 0.82).max(0.0));
        let alpha = (1.0 - progress).powi(2) * f32::from(local >= 0.0) * life_fade;
        background.0 = if particle.distance > 430.0 {
            Color::srgba(1.0, 0.40, 0.04, alpha)
        } else {
            Color::srgba(1.0, 0.86, 0.30, alpha)
        };
    }

    let title_local = elapsed - 0.45;
    let title_reveal = ease_out_cubic((title_local / 0.34).clamp(0.0, 1.0));
    let title_fade = ((2.10 - elapsed) / 0.42).clamp(0.0, 1.0);
    let title_alpha = title_reveal * title_fade;
    for (mut transform, mut background, mut border, mut shadow) in &mut visuals.p5() {
        transform.translation = Val2::px(0.0, 42.0 * (1.0 - title_reveal) - blast * 7.0);
        transform.scale = Vec2::splat(1.58 - title_reveal * 0.58 + blast * 0.03);
        background.0 = Color::srgba(0.10, 0.045, 0.008, title_alpha * 0.72);
        border.set_all(Color::srgba(1.0, 0.80, 0.28, title_alpha * 0.94));
        if let Some(style) = shadow.0.first_mut() {
            style.color = Color::srgba(1.0, 0.48, 0.06, title_alpha * 0.58);
            style.spread_radius = px(10.0 + title_reveal * 13.0);
            style.blur_radius = px(8.0 + title_reveal * 10.0);
        }
    }
    for mut color in &mut visuals.p6() {
        color.0 = Color::srgba(1.0, 0.84, 0.30, title_alpha);
    }
}

pub(super) fn add_heaven_bomb_effect(commands: &mut UiComposer, root: Entity, assets: &UiAssets) {
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
                    index as f32 / 14.0 * consts::PI + 0.035 * (index % 2) as f32,
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
        let angle = index as f32 / 36.0 * consts::TAU + (index % 5) as f32 * 0.031;
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
