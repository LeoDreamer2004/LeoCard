//! 七鬼五二三出牌表现与发牌音效调度。

use super::*;

pub(in crate::app) fn sync_play_effect(
    client: Option<Res<ClientResource>>,
    mut effect: ResMut<PlayEffectState>,
    mut ui: ResMut<UiState>,
    assets: Res<UiAssets>,
    mut commands: Commands,
) {
    let Some(model) = client.as_deref().map(|client| client.0.model()) else {
        if effect.active.take().is_some() {
            ui.dirty = true;
        }
        effect.seen_serial = 0;
        return;
    };
    if effect.seen_serial == model.play_effect_serial() {
        return;
    }
    effect.seen_serial = model.play_effect_serial();
    if let Some((_, play)) = model.last_play_effect() {
        let sounds = match card_play_sound_kind(&play.kind) {
            CardPlaySoundKind::Place => &assets.place_sounds,
            CardPlaySoundKind::Shove => &assets.shove_sounds,
        };
        if !sounds.is_empty() {
            let sound = sounds[fastrand::usize(..sounds.len())].clone();
            commands.spawn((AudioPlayer::new(sound), PlaybackSettings::DESPAWN));
        }
    }
    effect.active = model
        .last_play_effect()
        .filter(|(_, play)| {
            matches!(
                play.kind,
                PlayKind::Straight { .. }
                    | PlayKind::ConsecutivePairs { .. }
                    | PlayKind::Airplane { .. }
                    | PlayKind::Bomb(_)
                    | PlayKind::HeavenBomb
            )
        })
        .map(|(player, play)| ActivePlayEffect {
            player: *player,
            play: play.clone(),
            elapsed: 0.0,
            bomb_sound_played: false,
        });
    ui.dirty = true;
}

pub(in crate::app) fn card_play_sound_kind(kind: &PlayKind) -> CardPlaySoundKind {
    match kind {
        PlayKind::Straight { .. }
        | PlayKind::ConsecutivePairs { .. }
        | PlayKind::Airplane { .. } => CardPlaySoundKind::Shove,
        PlayKind::Single
        | PlayKind::Pair
        | PlayKind::Triple
        | PlayKind::TripleWithSingle
        | PlayKind::TripleWithPair
        | PlayKind::Bomb(_)
        | PlayKind::HeavenBomb => CardPlaySoundKind::Place,
    }
}

pub(in crate::app) fn advance_play_effect(
    time: Res<Time>,
    mut effect: ResMut<PlayEffectState>,
    mut roots: Query<&mut Visibility, With<PlayEffectRoot>>,
) {
    if effect.active.is_none() {
        for mut visibility in &mut roots {
            if *visibility != Visibility::Hidden {
                *visibility = Visibility::Hidden;
            }
        }
        return;
    }
    let Some(active) = effect.active.as_mut() else {
        unreachable!("active play effect was checked above");
    };
    active.elapsed += time.delta_secs();
    let elapsed = active.elapsed;
    let duration = match active.play.kind {
        PlayKind::HeavenBomb => 2.25,
        PlayKind::Bomb(_) => 1.35,
        PlayKind::Airplane { .. } => 1.38,
        _ => 1.18,
    };
    if elapsed >= duration {
        effect.active = None;
        for mut visibility in &mut roots {
            *visibility = Visibility::Hidden;
        }
        return;
    }
    for mut visibility in &mut roots {
        if *visibility != Visibility::Visible {
            *visibility = Visibility::Visible;
        }
    }
}

pub(in crate::app) fn sequence_effect_style(
    kind: &PlayKind,
) -> Option<(&'static str, Color, SequenceEffectMotif)> {
    match kind {
        PlayKind::Straight { .. } => Some(("顺子", STRAIGHT_EFFECT, SequenceEffectMotif::Wind)),
        PlayKind::ConsecutivePairs { .. } => Some((
            "连对",
            CONSECUTIVE_PAIRS_EFFECT,
            SequenceEffectMotif::Flower,
        )),
        PlayKind::Airplane { .. } => Some(("飞机", AIRPLANE_EFFECT, SequenceEffectMotif::Airplane)),
        _ => None,
    }
}

pub(in crate::app) fn animate_sequence_play_effect(
    effect: Res<PlayEffectState>,
    mut visuals: ParamSet<(
        Query<(&SequenceEffectCard, &mut UiTransform, &mut ImageNode)>,
        Query<(
            &SequenceGuideSegment,
            &mut UiTransform,
            &mut BackgroundColor,
        )>,
        Query<&mut UiTransform, With<SequenceEffectLabel>>,
        Query<(&SequenceEffectLabelPart, &mut TextColor)>,
        Query<(&SequenceWindStreak, &mut UiTransform, &mut BackgroundColor)>,
        Query<(&SequenceFlowerPart, &mut UiTransform, &mut BackgroundColor)>,
        Query<(&mut UiTransform, &mut ImageNode), With<SequenceAirplane>>,
        Query<(
            &SequenceAirplaneTrail,
            &mut UiTransform,
            &mut BackgroundColor,
        )>,
    )>,
) {
    let Some(active) = effect.active.as_ref() else {
        return;
    };
    let Some((_, effect_color, _)) = sequence_effect_style(&active.play.kind) else {
        return;
    };
    let elapsed = active.elapsed;
    let sequence_exit = if matches!(active.play.kind, PlayKind::Airplane { .. }) {
        ((elapsed - 0.88) / 0.38).clamp(0.0, 1.0)
    } else {
        ((elapsed - 0.68) / 0.36).clamp(0.0, 1.0)
    };
    for (card, mut transform, mut image) in &mut visuals.p0() {
        let progress = ((elapsed - card.index as f32 * 0.055) / 0.24).clamp(0.0, 1.0);
        let reveal = ease_out_cubic(progress);
        transform.translation = Val2::px(-34.0 * (1.0 - reveal), -5.0 * (1.0 - reveal));
        transform.scale = Vec2::splat(0.90 + reveal * 0.10);
        // These are the real cards in the player's play area, so they stay visible
        // after the sweep instead of fading like the old table-centre duplicate did.
        image.color = Color::srgba(1.0, 1.0, 1.0, reveal);
    }
    for (segment, mut transform, mut background) in &mut visuals.p1() {
        let sweep = ((elapsed - 0.08) / 0.40).clamp(0.0, 1.0);
        let threshold = segment.index as f32 / segment.count.max(1) as f32;
        let reveal = ((sweep - threshold) * segment.count as f32).clamp(0.0, 1.0);
        let fade = ((elapsed - 0.82) / 0.28).clamp(0.0, 1.0);
        transform.scale.x = reveal;
        background.0 = effect_color.with_alpha(reveal * (1.0 - fade) * 0.92);
    }
    let reveal = ease_out_cubic(((elapsed - 0.22) / 0.22).clamp(0.0, 1.0));
    let fade = sequence_exit;
    let alpha = reveal * (1.0 - fade);
    let scale = 0.86 + reveal * 0.14;
    let base = effect_color.to_srgba();
    let outline_color = Color::srgb(
        base.red + (1.0 - base.red) * 0.32,
        base.green + (1.0 - base.green) * 0.32,
        base.blue + (1.0 - base.blue) * 0.32,
    );
    for mut transform in &mut visuals.p2() {
        transform.translation = Val2::px(12.0 + fade * 62.0, 0.0);
        transform.scale = Vec2::new(scale * 1.08, scale);
    }
    for (part, mut color) in &mut visuals.p3() {
        color.0 = if part.outline {
            outline_color.with_alpha(alpha * 0.62)
        } else {
            effect_color.with_alpha(alpha)
        };
    }

    // Match the label's exit window so the motif reads as one coherent decoration.
    let motif_exit = sequence_exit;
    let motif_fade = 1.0 - motif_exit;
    let motif_exit_x = ease_out_cubic(motif_exit) * 58.0;
    for (streak, mut transform, mut background) in &mut visuals.p4() {
        let entry = ((elapsed - 0.10 - streak.index as f32 * 0.018) / 0.16).clamp(0.0, 1.0);
        let cycle = (elapsed * 1.65 + streak.index as f32 * 0.137).fract();
        let drift = ease_out_cubic(cycle);
        let wave = (cycle * std::f32::consts::TAU + streak.index as f32 * 0.9).sin();
        transform.translation = Val2::px(-18.0 + drift * 66.0 + motif_exit_x, wave * 2.8);
        transform.scale = Vec2::new(0.42 + (1.0 - cycle) * 0.72, 0.74 + wave.abs() * 0.26);
        let alpha = entry * motif_fade * (std::f32::consts::PI * cycle).sin().max(0.0);
        background.0 = effect_color.with_alpha(alpha * 0.82);
    }

    for (part, mut transform, mut background) in &mut visuals.p5() {
        let delay = if part.petal {
            part.index as f32 * 0.026
        } else {
            0.08
        };
        let progress = ((elapsed - 0.14 - delay) / 0.32).clamp(0.0, 1.0);
        // A small overshoot makes each petal visibly unfold instead of merely fading in.
        let overshoot = if progress < 1.0 {
            1.0 - (1.0 - progress).powi(2) * (1.0 - progress * 1.35)
        } else {
            1.0
        };
        let pulse = 1.0 + (elapsed * 9.0 + part.index as f32).sin() * 0.035 * progress;
        if part.petal {
            let angle =
                part.index as f32 / 7.0 * std::f32::consts::TAU - std::f32::consts::FRAC_PI_2;
            let radius = 9.5 * overshoot;
            transform.translation =
                Val2::px(angle.cos() * radius + motif_exit_x, angle.sin() * radius);
            transform.rotation = Rot2::radians(angle + std::f32::consts::FRAC_PI_2);
            transform.scale = Vec2::new(0.52 + overshoot * 0.48, overshoot * pulse);
            let tint = if part.index % 2 == 0 {
                Color::srgb(0.98, 0.43, 0.76)
            } else {
                Color::srgb(0.82, 0.53, 1.0)
            };
            background.0 = tint.with_alpha(progress * motif_fade * 0.95);
        } else {
            transform.translation = Val2::px(motif_exit_x, 0.0);
            transform.scale = Vec2::splat(overshoot * pulse);
            background.0 = Color::srgb(0.58, 0.96, 0.78).with_alpha(progress * motif_fade);
        }
    }

    for (mut transform, mut image) in &mut visuals.p6() {
        let progress = ((elapsed - 0.12) / 0.94).clamp(0.0, 1.0).powf(1.28);
        let pose = sequence_airplane_pose(progress);
        let appear = ((elapsed - 0.10) / 0.12).clamp(0.0, 1.0);
        transform.translation = Val2::px(pose.position.x + motif_exit_x, pose.position.y);
        transform.rotation = Rot2::radians(pose.rotation);
        transform.scale = Vec2::splat(0.50 + progress * 0.42);
        image.color = Color::WHITE.with_alpha(appear * motif_fade);
    }

    for (trail, mut transform, mut background) in &mut visuals.p7() {
        let delay = 0.055 + trail.index as f32 * 0.042;
        let progress = ((elapsed - 0.12 - delay) / 0.94).clamp(0.0, 1.0).powf(1.28);
        let pose = sequence_airplane_pose(progress);
        let direction = Vec2::new(pose.rotation.cos(), pose.rotation.sin());
        let normal = Vec2::new(-direction.y, direction.x);
        let behind = 13.0 + trail.index as f32 * 7.0;
        let lane = (trail.index as f32 - 1.5) * 1.2;
        let position = pose.position - direction * behind + normal * lane;
        let puff = ((elapsed - 0.12 - delay) / 0.18).clamp(0.0, 1.0);
        transform.translation = Val2::px(position.x + motif_exit_x, position.y);
        transform.rotation = Rot2::radians(pose.rotation);
        transform.scale = Vec2::new(0.5 + puff * 0.75, 0.45 + puff * 0.35);
        background.0 = effect_color.with_alpha(puff * motif_fade * 0.48);
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(in crate::app) struct SequenceAirplanePose {
    pub(in crate::app) position: Vec2,
    pub(in crate::app) rotation: f32,
}

pub(in crate::app) fn sequence_airplane_pose(progress: f32) -> SequenceAirplanePose {
    let progress = progress.clamp(0.0, 1.0);
    // Start horizontally below the title, then bend around its lower-right corner.
    let p0 = Vec2::new(-6.0, 20.0);
    let p1 = Vec2::new(20.0, 20.0);
    let p2 = Vec2::new(57.0, 13.0);
    let p3 = Vec2::new(84.0, -43.0);
    let inverse = 1.0 - progress;
    let position = p0 * inverse.powi(3)
        + p1 * (3.0 * inverse.powi(2) * progress)
        + p2 * (3.0 * inverse * progress.powi(2))
        + p3 * progress.powi(3);
    let tangent = (p1 - p0) * (3.0 * inverse.powi(2))
        + (p2 - p1) * (6.0 * inverse * progress)
        + (p3 - p2) * (3.0 * progress.powi(2));
    SequenceAirplanePose {
        position,
        rotation: tangent.y.atan2(tangent.x),
    }
}

pub(in crate::app) fn animate_bomb_play_effect(
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
    let explosion_at = if matches!(active.play.kind, PlayKind::HeavenBomb) {
        0.44
    } else {
        0.52
    };
    if matches!(active.play.kind, PlayKind::Bomb(_) | PlayKind::HeavenBomb)
        && elapsed >= explosion_at
        && !active.bomb_sound_played
    {
        commands.spawn((
            AudioPlayer::new(assets.bomb_explosion_sound.clone()),
            PlaybackSettings {
                volume: Volume::Linear(if matches!(active.play.kind, PlayKind::HeavenBomb) {
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
        let arc = (travel * std::f32::consts::PI).sin() * 86.0;
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

pub(in crate::app) fn animate_heaven_bomb_play_effect(
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
        .filter(|active| matches!(active.play.kind, PlayKind::HeavenBomb))
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

pub(in crate::app) fn queue_deal_animations(
    client: Option<Res<ClientResource>>,
    mut ui: ResMut<UiState>,
    assets: Res<UiAssets>,
    mut commands: Commands,
) {
    let Some(mut hand) = client
        .as_deref()
        .and_then(|client| client.0.model().qigui523_game())
        .map(|game| game.your_hand.clone())
    else {
        if !ui.observed_hand.is_empty() {
            ui.observed_hand.clear();
        }
        return;
    };
    sort_cards_high_to_low(&mut hand);

    let new_cards = hand
        .iter()
        .copied()
        .filter(|card| !ui.observed_hand.contains(card))
        .collect::<Vec<_>>();
    for (index, card) in new_cards.into_iter().enumerate() {
        let animation = ui.card_animations.entry(card).or_default();
        animation.deal_elapsed = -(index as f32 * 0.045);
        animation.dealing = true;
        if !assets.deal_sounds.is_empty() {
            commands.spawn(PendingDealSound {
                remaining: index as f32 * 0.045,
                variant: fastrand::usize(..assets.deal_sounds.len()),
            });
        }
    }
    if ui.observed_hand != hand {
        ui.observed_hand = hand;
    }
}
