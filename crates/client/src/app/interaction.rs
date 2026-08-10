//! Chat, social-item projectiles, score capture, and their persistent animations.

use super::*;

const SHENGJI_FAILED_THROW_RETURN_DURATION: f32 = 0.42;
const SHENGJI_THROW_PENALTY_DELAY: f32 = 0.38;
const SHENGJI_THROW_PENALTY_DURATION: f32 = 0.58;

#[derive(Clone, Copy, Debug)]
pub(super) struct ShengjiFailedThrowCardVisual {
    pub(super) translation: Vec2,
    pub(super) scale: f32,
    pub(super) rotation_radians: f32,
    pub(super) visible: bool,
}

pub(super) fn shengji_failed_throw_card_visual(
    stage: ShengjiThrowFailureStage,
    index: usize,
    count: usize,
    elapsed: f32,
    return_direction: Vec2,
) -> ShengjiFailedThrowCardVisual {
    let spread = index as f32 - count.saturating_sub(1) as f32 * 0.5;
    match stage {
        ShengjiThrowFailureStage::Showing => {
            // 先由一叠牌完整展开；停顿片刻后向两边裂开，并弹回原位。
            let reveal = ease_out_cubic((elapsed / 0.20).clamp(0.0, 1.0));
            let crack_progress = ((elapsed - 0.30) / 0.48).clamp(0.0, 1.0);
            let crack = (crack_progress * std::f32::consts::PI).sin();
            let side = spread.signum();
            let alternate = if index.is_multiple_of(2) { -1.0 } else { 1.0 };
            ShengjiFailedThrowCardVisual {
                translation: Vec2::new(
                    -spread * 24.0 * (1.0 - reveal) + side * (9.0 + spread.abs() * 1.8) * crack,
                    (alternate * 5.0 - 4.0) * crack,
                ),
                scale: 0.94 + reveal * 0.06 + crack * 0.035,
                rotation_radians: (spread * 1.6 * crack).to_radians(),
                visible: true,
            }
        }
        ShengjiThrowFailureStage::Returning => {
            let stagger = index as f32 / count.max(1) as f32 * 0.07;
            let raw = ((elapsed - stagger) / (SHENGJI_FAILED_THROW_RETURN_DURATION - 0.07))
                .clamp(0.0, 1.0);
            let return_progress = raw * raw * raw;
            let rebound = 1.0 - ease_out_cubic((raw / 0.34).clamp(0.0, 1.0));
            let alternate = if index.is_multiple_of(2) { -1.0 } else { 1.0 };
            let split = Vec2::new(spread.signum() * (7.0 + spread.abs()), alternate * 4.0);
            ShengjiFailedThrowCardVisual {
                translation: split * rebound + return_direction * return_progress,
                scale: 1.0 - return_progress * 0.22,
                rotation_radians: (spread * 1.1 * rebound).to_radians(),
                visible: raw < 1.0,
            }
        }
    }
}

pub(super) fn animate_shengji_failed_throw_cards(
    time: Res<Time>,
    mut cards: Query<(
        &mut ShengjiFailedThrowCard,
        &mut UiTransform,
        &mut Visibility,
    )>,
) {
    for (mut animation, mut transform, mut visibility) in &mut cards {
        animation.elapsed += time.delta_secs();
        let visual = shengji_failed_throw_card_visual(
            animation.stage,
            animation.index,
            animation.count,
            animation.elapsed,
            animation.direction,
        );
        transform.translation = Val2::px(visual.translation.x, visual.translation.y);
        transform.scale = Vec2::splat(visual.scale);
        transform.rotation = Rot2::radians(visual.rotation_radians);
        *visibility = if visual.visible {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }
}

pub(super) fn animate_shengji_failed_throw_labels(
    time: Res<Time>,
    mut labels: Query<(
        &mut ShengjiFailedThrowLabel,
        &mut UiTransform,
        &mut TextColor,
        &mut Visibility,
    )>,
) {
    for (mut animation, mut transform, mut color, mut visibility) in &mut labels {
        animation.elapsed += time.delta_secs();
        if animation.returning {
            let progress = ease_out_cubic((animation.elapsed / 0.30).clamp(0.0, 1.0));
            transform.translation = Val2::px(0.0, -12.0 * progress);
            transform.scale = Vec2::splat(1.0 - 0.12 * progress);
            color.0 = DANGER.with_alpha(1.0 - progress);
            if progress >= 1.0 {
                *visibility = Visibility::Hidden;
            }
        } else {
            let enter = ease_out_cubic((animation.elapsed / 0.16).clamp(0.0, 1.0));
            let shake_progress = ((animation.elapsed - 0.24) / 0.42).clamp(0.0, 1.0);
            let shake =
                (shake_progress * std::f32::consts::TAU * 3.0).sin() * (1.0 - shake_progress) * 3.0;
            transform.translation = Val2::px(shake, 7.0 * (1.0 - enter));
            transform.scale = Vec2::splat(0.86 + 0.14 * enter);
            color.0 = DANGER.with_alpha(enter);
            *visibility = Visibility::Visible;
        }
    }
}

pub(super) fn animate_shengji_throw_penalty_floats(
    time: Res<Time>,
    mut penalties: Query<(
        &mut ShengjiThrowPenaltyFloat,
        &mut Node,
        &mut UiTransform,
        &mut TextColor,
        &mut Visibility,
    )>,
) {
    for (mut animation, mut node, mut transform, mut color, mut visibility) in &mut penalties {
        animation.elapsed += time.delta_secs();
        let raw = ((animation.elapsed - SHENGJI_THROW_PENALTY_DELAY)
            / SHENGJI_THROW_PENALTY_DURATION)
            .clamp(0.0, 1.0);
        let travel = ease_out_cubic(raw);
        let position = animation.source.lerp(animation.target, travel);
        node.left = percent(position.x);
        node.top = percent(position.y);
        let arc = (raw * std::f32::consts::PI).sin();
        transform.translation = Val2::px(-30.0, -18.0 - arc * 20.0);
        transform.scale = Vec2::splat(0.86 + arc * 0.20);
        color.0 =
            DANGER.with_alpha((raw / 0.12).clamp(0.0, 1.0) * ((1.0 - raw) / 0.14).clamp(0.0, 1.0));
        *visibility = if animation.elapsed < SHENGJI_THROW_PENALTY_DELAY || raw >= 1.0 {
            Visibility::Hidden
        } else {
            Visibility::Visible
        };
    }
}

pub(super) fn animate_shengji_throw_penalty_score_pulses(
    time: Res<Time>,
    mut scores: Query<(&mut ShengjiThrowPenaltyScorePulse, &mut UiTransform)>,
) {
    for (mut animation, mut transform) in &mut scores {
        animation.elapsed += time.delta_secs();
        let progress = ((animation.elapsed
            - SHENGJI_THROW_PENALTY_DELAY
            - SHENGJI_THROW_PENALTY_DURATION * 0.72)
            / 0.30)
            .clamp(0.0, 1.0);
        transform.scale = Vec2::splat(1.0 + (progress * std::f32::consts::PI).sin() * 0.18);
    }
}

fn interaction_anchor_in_layer(
    player: PlayerId,
    layer_node: &ComputedNode,
    layer_transform: &UiGlobalTransform,
    avatars: &Query<(&PlayerAvatarAnchor, &ComputedNode, &UiGlobalTransform)>,
) -> Option<Vec2> {
    let (_, avatar_node, avatar_transform) =
        avatars.iter().find(|(anchor, _, _)| anchor.0 == player)?;
    let inverse = layer_transform.try_inverse()?;
    let avatar_center = avatar_transform.to_scale_angle_translation().2;
    let mut position = inverse.transform_point2(avatar_center) + layer_node.size() * 0.5;
    position.y += avatar_node.size().y * 0.16;
    Some(position * layer_node.inverse_scale_factor())
}

#[derive(Clone, Copy)]
struct ScoreAnchor {
    center: Vec2,
    gain_left: f32,
}

fn score_anchor_in_layer(
    player: PlayerId,
    layer_node: &ComputedNode,
    layer_transform: &UiGlobalTransform,
    anchors: &Query<(&PlayerGameScoreText, &ComputedNode, &UiGlobalTransform)>,
) -> Option<ScoreAnchor> {
    let (marker, score_node, score_transform) = anchors
        .iter()
        .find(|(anchor, _, _)| anchor.player() == player)?;
    let inverse = layer_transform.try_inverse()?;
    let center = score_transform.to_scale_angle_translation().2;
    let center = (inverse.transform_point2(center) + layer_node.size() * 0.5)
        * layer_node.inverse_scale_factor();
    let size = score_node.size() * score_node.inverse_scale_factor();
    Some(ScoreAnchor {
        center,
        gain_left: marker.gain_left(center.x, size.x),
    })
}

fn score_source_in_layer(
    player: PlayerId,
    layer_node: &ComputedNode,
    layer_transform: &UiGlobalTransform,
    sources: &Query<(&FinishedHandScoreSource, &ComputedNode, &UiGlobalTransform)>,
) -> Option<Vec2> {
    let (_, source_node, source_transform) =
        sources.iter().find(|(source, _, _)| source.0 == player)?;
    let inverse = layer_transform.try_inverse()?;
    let center = source_transform.to_scale_angle_translation().2;
    let mut center = inverse.transform_point2(center) + layer_node.size() * 0.5;
    center.y += source_node.size().y * 0.08;
    Some(center * layer_node.inverse_scale_factor())
}

fn interaction_target_offset(kind: PlayerInteractionKind, seed: u32) -> Vec2 {
    if matches!(kind, PlayerInteractionKind::Wine) {
        return Vec2::ZERO;
    }
    let x = (seed & 0xff) as f32 / 255.0;
    let y = ((seed >> 8) & 0xff) as f32 / 255.0;
    Vec2::new((x - 0.5) * 9.0, (y - 0.5) * 6.0)
}

fn interaction_rotates(kind: PlayerInteractionKind) -> bool {
    matches!(kind, PlayerInteractionKind::Shoe)
}

#[allow(clippy::too_many_arguments)]
fn spawn_player_interaction_projectile(
    commands: &mut Commands,
    layer: Entity,
    assets: &UiAssets,
    source: Vec2,
    target: Vec2,
    kind: PlayerInteractionKind,
    sound_variant: u8,
    play_sound: bool,
    delay: f32,
    appear_duration: f32,
    travel_duration: f32,
    impact_duration: f32,
) {
    let effect = commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                left: px(source.x - 30.0),
                top: px(source.y - 30.0),
                width: px(60),
                height: px(60),
                ..default()
            },
            ImageNode::new(
                assets
                    .interaction_images
                    .get(&(kind, false))
                    .expect("every interaction has a flight image")
                    .clone(),
            )
            .with_color(Color::WHITE.with_alpha(0.0)),
            UiTransform::default(),
            ActivePlayerInteraction {
                source,
                target,
                kind,
                sound_variant,
                play_sound,
                elapsed: -delay,
                appear_duration,
                travel_duration,
                impact_duration,
                launched: false,
                impacted: false,
            },
            GlobalZIndex(1200),
            FocusPolicy::Pass,
        ))
        .id();
    commands.entity(layer).add_child(effect);
}

pub(super) fn interaction_volley_lane(index: usize) -> f32 {
    const LANES: [f32; 10] = [-2.0, 2.0, -1.0, 1.0, 0.0, -1.5, 1.5, -0.5, 0.5, 0.0];
    LANES[index] * 9.0
}

pub(super) fn interaction_volley_kind(
    kind: PlayerInteractionKind,
) -> Option<PlayerInteractionKind> {
    match kind {
        PlayerInteractionKind::Wine => Some(PlayerInteractionKind::Flower),
        PlayerInteractionKind::Shoe => Some(PlayerInteractionKind::Egg),
        PlayerInteractionKind::Flower | PlayerInteractionKind::Egg => None,
    }
}

pub(super) fn interaction_volley_interval(kind: PlayerInteractionKind) -> f32 {
    match kind {
        PlayerInteractionKind::Egg => 0.09,
        PlayerInteractionKind::Flower => 0.055,
        PlayerInteractionKind::Wine | PlayerInteractionKind::Shoe => 0.0,
    }
}

pub(super) fn sync_player_interactions(
    mut commands: Commands,
    mut client: Option<ResMut<ClientResource>>,
    assets: Res<UiAssets>,
    layers: Query<(Entity, &ComputedNode, &UiGlobalTransform), With<PlayerInteractionLayer>>,
    avatars: Query<(&PlayerAvatarAnchor, &ComputedNode, &UiGlobalTransform)>,
) {
    let Ok((layer, layer_node, layer_transform)) = layers.single() else {
        return;
    };
    let Some(client) = client.as_deref_mut() else {
        return;
    };
    let interactions = client.0.take_player_interactions();
    if interactions.is_empty() {
        return;
    }
    for interaction in interactions {
        let Some(source) =
            interaction_anchor_in_layer(interaction.source, layer_node, layer_transform, &avatars)
        else {
            continue;
        };
        let Some(target) =
            interaction_anchor_in_layer(interaction.target, layer_node, layer_transform, &avatars)
        else {
            continue;
        };
        let target = target + interaction_target_offset(interaction.kind, interaction.seed);
        let sound_variant = (interaction.seed & 1) as u8;
        if let Some(volley_kind) = interaction_volley_kind(interaction.kind) {
            let volley_interval = interaction_volley_interval(volley_kind);
            let direction = (target - source).normalize_or_zero();
            let perpendicular = Vec2::new(-direction.y, direction.x);
            for index in 0..10 {
                let lane = interaction_volley_lane(index);
                let source_offset = perpendicular * lane;
                let target_offset = perpendicular * (lane * 0.72);
                spawn_player_interaction_projectile(
                    &mut commands,
                    layer,
                    &assets,
                    source + source_offset,
                    target + target_offset,
                    volley_kind,
                    ((interaction.seed >> (index % 16)) & 1) as u8,
                    !matches!(interaction.kind, PlayerInteractionKind::Wine) || index == 0,
                    index as f32 * volley_interval,
                    0.07,
                    0.40,
                    0.42,
                );
            }
            spawn_player_interaction_projectile(
                &mut commands,
                layer,
                &assets,
                source,
                target,
                interaction.kind,
                sound_variant,
                true,
                9.0 * volley_interval + 0.55,
                0.16,
                HEAVY_INTERACTION_TRAVEL_DURATION,
                INTERACTION_IMPACT_DURATION,
            );
        } else {
            spawn_player_interaction_projectile(
                &mut commands,
                layer,
                &assets,
                source,
                target,
                interaction.kind,
                sound_variant,
                true,
                0.0,
                INTERACTION_APPEAR_DURATION,
                INTERACTION_TRAVEL_DURATION,
                INTERACTION_IMPACT_DURATION,
            );
        }
    }
}

pub(super) fn sync_score_capture_effect(
    mut commands: Commands,
    client: Option<Res<ClientResource>>,
    assets: Res<UiAssets>,
    mut state: ResMut<ScoreCaptureEffectState>,
    layers: Query<(Entity, &ComputedNode, &UiGlobalTransform), With<PlayerInteractionLayer>>,
    score_anchors: Query<(&PlayerGameScoreText, &ComputedNode, &UiGlobalTransform)>,
    source_anchors: Query<(&FinishedHandScoreSource, &ComputedNode, &UiGlobalTransform)>,
) {
    let Some(model) = client.as_deref().map(|client| client.0.model()) else {
        state.seen_serial = 0;
        state.active = None;
        return;
    };
    let serial = model.score_capture_serial();
    if serial == 0 {
        state.seen_serial = 0;
        state.active = None;
        return;
    }
    if serial == state.seen_serial {
        return;
    }
    let Some(capture) = model.last_score_capture().cloned() else {
        return;
    };
    let Ok((layer, layer_node, layer_transform)) = layers.single() else {
        return;
    };
    let Some(anchor) =
        score_anchor_in_layer(capture.player, layer_node, layer_transform, &score_anchors)
    else {
        return;
    };
    let mut sources = HashMap::new();
    for player in capture.source_players.iter().flatten().copied() {
        if sources.contains_key(&player) {
            continue;
        }
        let Some(source) =
            score_source_in_layer(player, layer_node, layer_transform, &source_anchors)
        else {
            // 终局界面刚重建时，公开手牌的布局要到下一帧才可用于定位。
            return;
        };
        sources.insert(player, source);
    }
    state.seen_serial = serial;
    state.active = Some(ActiveScoreCapture {
        capture: capture.clone(),
        elapsed: 0.0,
    });
    spawn_score_capture_effect(
        &mut commands,
        layer,
        layer_node.size() * layer_node.inverse_scale_factor(),
        anchor,
        &capture,
        &sources,
        &assets,
    );
}

fn spawn_score_capture_effect(
    commands: &mut Commands,
    layer: Entity,
    layer_size: Vec2,
    anchor: ScoreAnchor,
    capture: &ScoreCaptureEffect,
    source_anchors: &HashMap<PlayerId, Vec2>,
    assets: &UiAssets,
) {
    let source_center = Vec2::new(layer_size.x * 0.5, layer_size.y * 0.50);
    let target = anchor.center;
    let direction = target - source_center;
    if direction.length() > 1.0 {
        let vortex = commands
            .spawn((
                Node {
                    position_type: PositionType::Absolute,
                    left: px(target.x - 18.0),
                    top: px(target.y - 18.0),
                    width: px(36),
                    height: px(36),
                    border: UiRect::all(px(3)),
                    border_radius: BorderRadius::all(percent(50)),
                    ..default()
                },
                BackgroundColor(Color::NONE),
                BorderColor::all(Color::NONE),
                UiTransform::IDENTITY,
                ActiveScoreVortex { elapsed: 0.0 },
                GlobalZIndex(1550),
                FocusPolicy::Pass,
            ))
            .id();
        commands.entity(layer).add_child(vortex);
    }

    let card_count = capture.cards.len();
    for (index, card) in capture.cards.iter().copied().enumerate() {
        let source_player = capture.source_players.get(index).copied().flatten();
        let group_count = capture
            .source_players
            .iter()
            .filter(|source| **source == source_player)
            .count();
        let group_index = capture.source_players[..index]
            .iter()
            .filter(|source| **source == source_player)
            .count();
        let total_width = TABLE_SCORE_CARD_REVEAL * group_count.saturating_sub(1) as f32
            + CardSize::TableScore.dimensions().0;
        let offset_x = group_index as f32 * TABLE_SCORE_CARD_REVEAL - total_width * 0.5;
        let group_center = source_player
            .and_then(|player| source_anchors.get(&player).copied())
            .unwrap_or(source_center);
        let card_source =
            group_center + Vec2::new(offset_x, -CardSize::TableScore.dimensions().1 * 0.5);
        let image = assets
            .cards
            .get(&(card.rank(), card.suit()))
            .expect("all valid card faces are preloaded")
            .clone();
        let entity = commands
            .spawn((
                Node {
                    position_type: PositionType::Absolute,
                    left: px(card_source.x),
                    top: px(card_source.y),
                    width: px(CardSize::TableScore.dimensions().0),
                    height: px(CardSize::TableScore.dimensions().1),
                    border_radius: BorderRadius::all(px(4)),
                    ..default()
                },
                ImageNode::new(image),
                UiTransform::IDENTITY,
                ActiveScoreCaptureCard {
                    source: card_source,
                    target: target
                        - Vec2::new(
                            CardSize::TableScore.dimensions().0 * 0.5,
                            CardSize::TableScore.dimensions().1 * 0.5,
                        ),
                    elapsed: 0.0,
                    delay: if card_count > 1 {
                        index as f32 / (card_count - 1) as f32 * 0.055
                    } else {
                        0.0
                    },
                    curve: if index % 2 == 0 { 1.0 } else { -1.0 }
                        * (0.72 + index as f32 % 3.0 * 0.14),
                },
                GlobalZIndex(1560 + index as i32),
                FocusPolicy::Pass,
            ))
            .id();
        commands.entity(layer).add_child(entity);
    }

    let score = spawn_node(
        commands,
        layer,
        Node {
            position_type: PositionType::Absolute,
            left: px(anchor.gain_left),
            top: px(target.y - 20.0),
            width: px(76),
            height: px(36),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::FlexStart,
            ..default()
        },
        None,
    );
    commands
        .entity(score)
        .insert((UiTransform::IDENTITY, GlobalZIndex(1700), FocusPolicy::Pass));
    let text = add_text(
        commands,
        score,
        format!(
            "+{}",
            capture.score_after.saturating_sub(capture.score_before)
        ),
        22.0,
        Color::NONE,
        assets,
    );
    commands
        .entity(score)
        .insert(ActiveScoreGainText { elapsed: 0.0, text });
}

pub(super) fn animate_score_capture_effects(
    time: Res<Time>,
    mut commands: Commands,
    mut state: ResMut<ScoreCaptureEffectState>,
    mut effects: ParamSet<(
        Query<(
            Entity,
            &mut ActiveScoreCaptureCard,
            &mut UiTransform,
            &mut ImageNode,
        )>,
        Query<(
            Entity,
            &mut ActiveScoreVortex,
            &mut BackgroundColor,
            &mut BorderColor,
            &mut UiTransform,
        )>,
        Query<(Entity, &mut ActiveScoreGainText, &mut UiTransform)>,
    )>,
    mut texts: Query<(Option<&PlayerGameScoreText>, &mut Text, &mut TextColor)>,
) {
    if let Some(active) = state.active.as_mut() {
        active.elapsed += time.delta_secs();
        let displayed = rolling_captured_score(&active.capture, active.elapsed);
        for (marker, mut text, _) in &mut texts {
            if let Some(marker) = marker
                && marker.player() == active.capture.player
            {
                text.0 = marker.label(displayed);
            }
        }
        if active.elapsed >= SCORE_ROLL_DELAY + SCORE_ROLL_DURATION + 0.30 {
            state.active = None;
        }
    }

    for (entity, mut card, mut transform, mut image) in &mut effects.p0() {
        card.elapsed += time.delta_secs();
        let local = card.elapsed - card.delay;
        if local < 0.0 {
            image.color = Color::NONE;
            continue;
        }
        let progress = (local / SCORE_CAPTURE_TRAVEL_DURATION).clamp(0.0, 1.0);
        let pose = vortex_card_pose(card.source, card.target, progress, card.curve);
        transform.translation = Val2::px(
            pose.position.x - card.source.x,
            pose.position.y - card.source.y,
        );
        transform.rotation = Rot2::radians(pose.rotation);
        transform.scale = pose.scale;
        image.color = Color::WHITE.with_alpha(pose.opacity);
        if progress >= 1.0 {
            commands.entity(entity).despawn();
        }
    }

    for (entity, mut vortex, mut background, mut border, mut transform) in &mut effects.p1() {
        vortex.elapsed += time.delta_secs();
        let duration = SCORE_CAPTURE_TRAVEL_DURATION + 0.16;
        let progress = (vortex.elapsed / duration).clamp(0.0, 1.0);
        let enter = (progress / 0.14).clamp(0.0, 1.0);
        let exit = ((1.0 - progress) / 0.30).clamp(0.0, 1.0);
        let alpha = enter * exit;
        background.0 = Color::BLACK.with_alpha(alpha * 0.82);
        border.set_all(Color::srgb(0.46, 1.0, 0.72).with_alpha(alpha * 0.82));
        transform.rotation = Rot2::radians(progress * std::f32::consts::TAU * 1.7);
        transform.scale = Vec2::splat(0.46 + (progress * std::f32::consts::PI).sin() * 0.72);
        if progress >= 1.0 {
            commands.entity(entity).despawn();
        }
    }

    for (entity, mut gain, mut transform) in &mut effects.p2() {
        gain.elapsed += time.delta_secs();
        let local = gain.elapsed - SCORE_ROLL_DELAY;
        if local < 0.0 {
            continue;
        }
        let progress = (local / (SCORE_ROLL_DURATION + 0.30)).clamp(0.0, 1.0);
        let fade = ((SCORE_ROLL_DURATION + 0.30 - local) / 0.30).clamp(0.0, 1.0);
        transform.translation = Val2::px(0.0, -22.0 * ease_out_cubic(progress));
        transform.scale = Vec2::splat(0.82 + 0.18 * ease_out_cubic((progress * 4.0).min(1.0)));
        if let Ok((_, _, mut color)) = texts.get_mut(gain.text) {
            color.0 = ACCENT.with_alpha(fade);
        }
        if local >= SCORE_ROLL_DURATION + 0.30 {
            commands.entity(entity).despawn();
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct VortexCardPose {
    pub(super) position: Vec2,
    pub(super) rotation: f32,
    pub(super) scale: Vec2,
    pub(super) opacity: f32,
}

pub(super) fn vortex_card_pose(
    source: Vec2,
    target: Vec2,
    progress: f32,
    curve: f32,
) -> VortexCardPose {
    let progress = progress.clamp(0.0, 1.0);
    let position_at = |progress: f32| {
        // 近似引力加速：时间幂函数让牌在接近“漩涡”时明显提速；半径按幂次
        // 收缩，同时施加有限角位移，形成不会绕桌一整圈的吸入螺旋。
        let accelerated = progress.clamp(0.0, 1.0).powf(1.72);
        let radius = (1.0 - accelerated).powf(1.18);
        let angle = curve * 1.34 * accelerated;
        let (sin, cos) = angle.sin_cos();
        let offset = source - target;
        let rotated = Vec2::new(
            offset.x * cos - offset.y * sin,
            offset.x * sin + offset.y * cos,
        );
        target + rotated * radius
    };
    let position = position_at(progress);
    let next = position_at((progress + 0.002).min(1.0));
    let tangent = (next - position).normalize_or_zero();
    let accelerated = progress.powf(1.72);
    // 潮汐形变近似：靠近目标后沿运动方向拉长，垂直方向急剧收窄；再乘上
    // 整体坍缩系数，使长边最终也收敛为零，形成“尖端被吸入”的观感。
    let tidal_input = ((accelerated - 0.48) / 0.52).clamp(0.0, 1.0);
    let tidal = tidal_input * tidal_input * (3.0 - 2.0 * tidal_input);
    let collapse = (1.0 - accelerated).powf(0.62);
    let scale = Vec2::new(
        collapse * (1.0 - tidal * 0.84),
        collapse * (1.0 + tidal * 1.34),
    );
    let rotation = if tangent.length_squared() > 0.0 {
        tangent.y.atan2(tangent.x) - std::f32::consts::FRAC_PI_2
    } else {
        0.0
    };
    VortexCardPose {
        position,
        rotation,
        scale,
        opacity: ((1.0 - progress) / 0.075).clamp(0.0, 1.0),
    }
}

pub(super) fn rolling_captured_score(capture: &ScoreCaptureEffect, elapsed: f32) -> u32 {
    let progress = ((elapsed - SCORE_ROLL_DELAY) / SCORE_ROLL_DURATION).clamp(0.0, 1.0);
    let eased = ease_out_cubic(progress);
    (capture.score_before as f32
        + capture.score_after.saturating_sub(capture.score_before) as f32 * eased)
        .round() as u32
}

pub(super) fn displayed_captured_score(
    state: &ScoreCaptureEffectState,
    player: PlayerId,
    fallback: u32,
) -> u32 {
    state
        .active
        .as_ref()
        .filter(|active| active.capture.player == player)
        .map_or(fallback, |active| {
            rolling_captured_score(&active.capture, active.elapsed)
        })
}

pub(super) fn sync_shengji_score_capture_effect(
    mut commands: Commands,
    client: Option<Res<ClientResource>>,
    assets: Res<UiAssets>,
    mut state: ResMut<ShengjiScoreCaptureEffectState>,
    layers: Query<(Entity, &ComputedNode, &UiGlobalTransform), With<PlayerInteractionLayer>>,
    anchors: Query<(&ComputedNode, &UiGlobalTransform), With<ShengjiScoreTrayAnchor>>,
) {
    let Some(model) = client.as_deref().map(|client| client.0.model()) else {
        *state = ShengjiScoreCaptureEffectState::default();
        return;
    };
    let serial = model.shengji_score_capture_serial();
    if serial == 0 || serial == state.seen_serial {
        return;
    }
    let Some(capture) = model.last_shengji_score_capture().cloned() else {
        return;
    };
    let Ok((layer, layer_node, layer_transform)) = layers.single() else {
        return;
    };
    let Ok((anchor_node, anchor_transform)) = anchors.single() else {
        return;
    };
    let Some(inverse) = layer_transform.try_inverse() else {
        return;
    };
    let target = (inverse.transform_point2(anchor_transform.to_scale_angle_translation().2)
        + layer_node.size() * 0.5)
        * layer_node.inverse_scale_factor();
    let layer_size = layer_node.size() * layer_node.inverse_scale_factor();
    let source = Vec2::new(layer_size.x * 0.5, layer_size.y * 0.50);
    let gain_left =
        target.x + anchor_node.size().x * anchor_node.inverse_scale_factor() * 0.5 + 8.0;

    state.seen_serial = serial;
    state.active = Some(ActiveShengjiScoreCapture {
        capture: capture.clone(),
        elapsed: 0.0,
    });
    spawn_shengji_score_capture(
        &mut commands,
        layer,
        source,
        target,
        gain_left,
        &capture,
        &assets,
    );
}

fn spawn_shengji_score_capture(
    commands: &mut Commands,
    layer: Entity,
    source: Vec2,
    target: Vec2,
    gain_left: f32,
    capture: &ShengjiScoreCaptureEffect,
    assets: &UiAssets,
) {
    let vortex = commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                left: px(target.x - 18.0),
                top: px(target.y - 18.0),
                width: px(36),
                height: px(36),
                border: UiRect::all(px(3)),
                border_radius: BorderRadius::all(percent(50)),
                ..default()
            },
            BackgroundColor(Color::NONE),
            BorderColor::all(Color::NONE),
            UiTransform::IDENTITY,
            ActiveScoreVortex { elapsed: 0.0 },
            GlobalZIndex(1550),
            FocusPolicy::Pass,
        ))
        .id();
    commands.entity(layer).add_child(vortex);

    let (width, height) = (36.0, 49.0);
    let count = capture.cards.len();
    for (index, card) in capture.cards.iter().copied().enumerate() {
        let total_width = TABLE_SCORE_CARD_REVEAL * count.saturating_sub(1) as f32 + width;
        let card_source = source
            + Vec2::new(
                index as f32 * TABLE_SCORE_CARD_REVEAL - total_width * 0.5,
                -height * 0.5,
            );
        let entity = commands
            .spawn((
                Node {
                    position_type: PositionType::Absolute,
                    left: px(card_source.x),
                    top: px(card_source.y),
                    width: px(width),
                    height: px(height),
                    border_radius: BorderRadius::all(px(4)),
                    ..default()
                },
                ImageNode::new(shengji_card_face(card, assets)),
                UiTransform::IDENTITY,
                ActiveScoreCaptureCard {
                    source: card_source,
                    target: target - Vec2::new(width * 0.5, height * 0.5),
                    elapsed: 0.0,
                    delay: if count > 1 {
                        index as f32 / (count - 1) as f32 * 0.055
                    } else {
                        0.0
                    },
                    curve: if index % 2 == 0 { 1.0 } else { -1.0 }
                        * (0.72 + index as f32 % 3.0 * 0.14),
                },
                GlobalZIndex(1560 + index as i32),
                FocusPolicy::Pass,
            ))
            .id();
        commands.entity(layer).add_child(entity);
    }

    let gain = spawn_node(
        commands,
        layer,
        Node {
            position_type: PositionType::Absolute,
            left: px(gain_left),
            top: px(target.y - 20.0),
            width: px(76),
            height: px(36),
            align_items: AlignItems::Center,
            ..default()
        },
        None,
    );
    commands
        .entity(gain)
        .insert((UiTransform::IDENTITY, GlobalZIndex(1700), FocusPolicy::Pass));
    let text = add_text(
        commands,
        gain,
        format!(
            "+{}",
            capture.score_after.saturating_sub(capture.score_before)
        ),
        22.0,
        Color::NONE,
        assets,
    );
    commands
        .entity(gain)
        .insert(ActiveScoreGainText { elapsed: 0.0, text });
}

pub(super) fn animate_shengji_score_capture_score(
    time: Res<Time>,
    mut state: ResMut<ShengjiScoreCaptureEffectState>,
    mut texts: Query<&mut Text, With<ShengjiCollectingScoreText>>,
) {
    let Some(active) = state.active.as_mut() else {
        return;
    };
    active.elapsed += time.delta_secs();
    let displayed = rolling_shengji_captured_score(&active.capture, active.elapsed);
    for mut text in &mut texts {
        text.0 = displayed.to_string();
    }
    if active.elapsed >= SCORE_ROLL_DELAY + SCORE_ROLL_DURATION + 0.30 {
        state.active = None;
    }
}

pub(super) fn rolling_shengji_captured_score(
    capture: &ShengjiScoreCaptureEffect,
    elapsed: f32,
) -> u32 {
    let progress = ((elapsed - SCORE_ROLL_DELAY) / SCORE_ROLL_DURATION).clamp(0.0, 1.0);
    let eased = ease_out_cubic(progress);
    (capture.score_before as f32
        + capture.score_after.saturating_sub(capture.score_before) as f32 * eased)
        .round() as u32
}

pub(super) fn displayed_shengji_captured_score(
    state: &ShengjiScoreCaptureEffectState,
    fallback: u32,
) -> u32 {
    state.active.as_ref().map_or(fallback, |active| {
        rolling_shengji_captured_score(&active.capture, active.elapsed)
    })
}

pub(super) fn update_shengji_settlement_animation(
    mut commands: Commands,
    time: Res<Time>,
    client: Option<Res<ClientResource>>,
    assets: Res<UiAssets>,
    mut animation: ResMut<ShengjiSettlementAnimation>,
) {
    let finished = client.as_deref().and_then(|client| {
        let game = client.0.model().shengji_game()?;
        let ShengjiPhaseView::Finished { result, .. } = &game.phase else {
            return None;
        };
        let nonnegative = result
            .reference_changes
            .iter()
            .find(|change| change.player == game.you)
            .is_none_or(|change| change.delta >= 0);
        Some((
            result.settlement_id,
            nonnegative,
            result.reference_changes.len(),
        ))
    });
    let Some((settlement_id, nonnegative, player_count)) = finished else {
        if animation.settlement_id.is_some() {
            *animation = ShengjiSettlementAnimation::default();
        }
        return;
    };

    if animation.settlement_id != Some(settlement_id) {
        *animation = ShengjiSettlementAnimation {
            settlement_id: Some(settlement_id),
            ..default()
        };
    } else {
        let duration = SHENGJI_SETTLEMENT_MODAL_DELAY
            + 0.46
            + player_count as f32 * SHENGJI_SETTLEMENT_ROW_INTERVAL
            + 0.80;
        animation.elapsed = (animation.elapsed + time.delta_secs()).min(duration);
    }

    if animation.elapsed >= SHENGJI_SETTLEMENT_MODAL_DELAY && !animation.outcome_sound_played {
        let sound = if nonnegative {
            assets.summary_score_sound.clone()
        } else {
            assets.summary_die_sound.clone()
        };
        commands.spawn((AudioPlayer::new(sound), PlaybackSettings::DESPAWN));
        animation.outcome_sound_played = true;
    }
}

pub(super) fn animate_shengji_settlement_visuals(
    animation: Res<ShengjiSettlementAnimation>,
    mut visuals: ParamSet<(
        Query<(
            &ShengjiKittyRevealCard,
            &mut UiTransform,
            &mut ImageNode,
            &mut Visibility,
        )>,
        Query<(&ShengjiTimedReveal, &mut Visibility)>,
        Query<
            (&ShengjiKittyScoreText, &mut Text, &mut UiTransform),
            Without<ShengjiSettlementTotalText>,
        >,
        Query<
            (&ShengjiSettlementTotalText, &mut Text, &mut UiTransform),
            Without<ShengjiKittyScoreText>,
        >,
        Query<(&mut UiTransform, &mut TextColor, &mut Visibility), With<ShengjiKittyMultiplier>>,
        Query<(&mut UiTransform, &mut Visibility), With<ShengjiSettlementModal>>,
        Query<
            (
                &ShengjiSettlementRow,
                &mut UiTransform,
                &mut BackgroundColor,
                &mut Visibility,
            ),
            Without<ShengjiSettlementModal>,
        >,
        Query<(&ShengjiSettlementActions, &mut Visibility)>,
    )>,
    mut panel_textures: Query<
        &mut ImageNode,
        (
            With<ShengjiSettlementPanelTexture>,
            Without<ShengjiKittyRevealCard>,
        ),
    >,
) {
    if !animation.is_changed() {
        return;
    }
    let elapsed = animation.elapsed;
    for (card, mut transform, mut image, mut visibility) in &mut visuals.p0() {
        let delay = card.index as f32 * SHENGJI_KITTY_CARD_INTERVAL;
        let raw = ((elapsed - delay) / SHENGJI_KITTY_CARD_ENTRY_DURATION).clamp(0.0, 1.0);
        let progress = ease_out_cubic(raw);
        *visibility = if raw > 0.0 {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
        transform.translation = Val2::px(0.0, 16.0 * (1.0 - progress));
        let bounce = if raw < 1.0 {
            (raw * std::f32::consts::PI).sin() * 0.08
        } else {
            0.0
        };
        transform.scale = Vec2::splat(0.78 + progress * 0.22 + bounce);
        image.color = Color::WHITE.with_alpha(progress);
    }
    for (reveal, mut visibility) in &mut visuals.p1() {
        *visibility = if elapsed >= reveal.delay {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }

    let multiplier_roll = ((elapsed
        - (SHENGJI_KITTY_MULTIPLIER_DELAY + SHENGJI_KITTY_MULTIPLIER_DURATION * 0.56))
        / (SHENGJI_KITTY_MULTIPLIER_DURATION * 0.44))
        .clamp(0.0, 1.0);
    for (score, mut text, mut transform) in &mut visuals.p2() {
        let target = if score.awarded == 0 {
            score.base
        } else {
            score.awarded
        };
        let displayed = (score.base as f32
            + (target.saturating_sub(score.base)) as f32 * ease_out_cubic(multiplier_roll))
        .round() as u32;
        text.0 = displayed.to_string();
        let pulse = (multiplier_roll * std::f32::consts::PI).sin() * 0.20;
        transform.scale = Vec2::splat(1.0 + pulse);
    }
    for (mut transform, mut color, mut visibility) in &mut visuals.p4() {
        let raw = ((elapsed - SHENGJI_KITTY_MULTIPLIER_DELAY) / SHENGJI_KITTY_MULTIPLIER_DURATION)
            .clamp(0.0, 1.0);
        *visibility = if raw > 0.0 && raw < 1.0 {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
        let progress = ease_out_cubic(raw);
        transform.translation = Val2::px(42.0 * (1.0 - progress), -24.0 * (1.0 - progress));
        transform.scale = Vec2::splat(0.82 + progress * 0.28);
        let alpha = ((1.0 - raw) / 0.24).clamp(0.0, 1.0);
        color.0 = READY.with_alpha(alpha);
    }

    let total_raw =
        ((elapsed - SHENGJI_TOTAL_ABSORB_DELAY) / SHENGJI_TOTAL_ABSORB_DURATION).clamp(0.0, 1.0);
    for (score, mut text, mut transform) in &mut visuals.p3() {
        let displayed = (score.target as f32 * ease_out_cubic(total_raw)).round() as u32;
        text.0 = displayed.to_string();
        let pulse = (total_raw * std::f32::consts::PI).sin() * 0.16;
        transform.scale = Vec2::splat(1.0 + pulse);
    }

    let modal_raw =
        ((elapsed - SHENGJI_SETTLEMENT_MODAL_DELAY) / SUMMARY_MODAL_ENTRY_DURATION).clamp(0.0, 1.0);
    let modal_progress = ease_out_cubic(modal_raw);
    for mut image in &mut panel_textures {
        image.color = Color::WHITE.with_alpha(0.98 * modal_progress);
    }
    for (mut transform, mut visibility) in &mut visuals.p5() {
        *visibility = if modal_raw > 0.0 {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
        transform.translation = Val2::px(0.0, 72.0 * (1.0 - modal_progress));
    }
    for (row, mut transform, mut background, mut visibility) in &mut visuals.p6() {
        let raw = ((elapsed - row.delay) / SUMMARY_ROW_ENTRY_DURATION).clamp(0.0, 1.0);
        let progress = ease_out_cubic(raw);
        *visibility = if raw > 0.0 {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
        transform.translation = Val2::px(0.0, 16.0 * (1.0 - progress));
        background.0 = PANEL_ALT.with_alpha(0.82 * progress);
    }
    for (actions, mut visibility) in &mut visuals.p7() {
        *visibility = if elapsed >= actions.delay {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }
}

pub(super) fn spawn_shengji_settlement_absorption(
    mut commands: Commands,
    assets: Res<UiAssets>,
    client: Option<Res<ClientResource>>,
    mut animation: ResMut<ShengjiSettlementAnimation>,
    layers: Query<(Entity, &ComputedNode, &UiGlobalTransform), With<PlayerInteractionLayer>>,
    tray: Query<(&ComputedNode, &UiGlobalTransform), With<ShengjiScoreTrayAnchor>>,
    kitty: Query<(&ComputedNode, &UiGlobalTransform), With<ShengjiKittyScoreAnchor>>,
    total: Query<(&ComputedNode, &UiGlobalTransform), With<ShengjiSettlementTotalAnchor>>,
) {
    if animation.absorption_spawned || animation.elapsed < SHENGJI_TOTAL_ABSORB_DELAY {
        return;
    }
    let Some(game) = client
        .as_deref()
        .and_then(|client| client.0.model().shengji_game())
    else {
        return;
    };
    let ShengjiPhaseView::Finished { result, .. } = &game.phase else {
        return;
    };
    let Ok((layer, layer_node, layer_transform)) = layers.single() else {
        return;
    };
    let (
        Ok((tray_node, tray_transform)),
        Ok((kitty_node, kitty_transform)),
        Ok((total_node, total_transform)),
    ) = (tray.single(), kitty.single(), total.single())
    else {
        return;
    };
    let Some(inverse) = layer_transform.try_inverse() else {
        return;
    };
    let center_in_layer = |_node: &ComputedNode, transform: &UiGlobalTransform| {
        (inverse.transform_point2(transform.to_scale_angle_translation().2)
            + layer_node.size() * 0.5)
            * layer_node.inverse_scale_factor()
    };
    let target = center_in_layer(total_node, total_transform);
    let pre_kitty = i64::from(result.trick_points)
        .saturating_add(i64::from(result.penalty_adjustment))
        .max(0) as u32;
    let kitty_award = u32::from(result.kitty_points).saturating_mul(result.kitty_multiplier);
    if pre_kitty > 0 {
        spawn_shengji_score_absorb(
            &mut commands,
            layer,
            center_in_layer(tray_node, tray_transform),
            target,
            pre_kitty,
            0.0,
            0.92,
            &assets,
        );
    }
    if kitty_award > 0 {
        spawn_shengji_score_absorb(
            &mut commands,
            layer,
            center_in_layer(kitty_node, kitty_transform),
            target,
            kitty_award,
            0.10,
            -0.82,
            &assets,
        );
    }
    let vortex = commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                left: px(target.x - 18.0),
                top: px(target.y - 18.0),
                width: px(36),
                height: px(36),
                border: UiRect::all(px(3)),
                border_radius: BorderRadius::all(percent(50)),
                ..default()
            },
            BackgroundColor(Color::NONE),
            BorderColor::all(Color::NONE),
            UiTransform::IDENTITY,
            ActiveScoreVortex { elapsed: 0.0 },
            GlobalZIndex(1670),
            FocusPolicy::Pass,
        ))
        .id();
    commands.entity(layer).add_child(vortex);
    animation.absorption_spawned = true;
}

fn spawn_shengji_score_absorb(
    commands: &mut Commands,
    layer: Entity,
    source_center: Vec2,
    target_center: Vec2,
    value: u32,
    delay: f32,
    curve: f32,
    assets: &UiAssets,
) {
    let size = Vec2::new(84.0, 34.0);
    let source = source_center - size * 0.5;
    let target = target_center - size * 0.5;
    let entity = add_text(commands, layer, format!("+{value}"), 23.0, ACCENT, assets);
    commands.entity(entity).insert((
        Node {
            position_type: PositionType::Absolute,
            left: px(source.x),
            top: px(source.y),
            width: px(size.x),
            height: px(size.y),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            ..default()
        },
        UiTransform::IDENTITY,
        ActiveShengjiScoreAbsorb {
            source,
            target,
            elapsed: 0.0,
            delay,
            curve,
        },
        GlobalZIndex(1680),
        FocusPolicy::Pass,
    ));
}

pub(super) fn animate_shengji_score_absorbs(
    mut commands: Commands,
    time: Res<Time>,
    mut effects: Query<(
        Entity,
        &mut ActiveShengjiScoreAbsorb,
        &mut UiTransform,
        &mut TextColor,
    )>,
) {
    for (entity, mut effect, mut transform, mut color) in &mut effects {
        effect.elapsed += time.delta_secs();
        let local = effect.elapsed - effect.delay;
        if local < 0.0 {
            color.0 = Color::NONE;
            continue;
        }
        let progress = (local / SHENGJI_TOTAL_ABSORB_DURATION).clamp(0.0, 1.0);
        let pose = vortex_card_pose(effect.source, effect.target, progress, effect.curve);
        transform.translation = Val2::px(
            pose.position.x - effect.source.x,
            pose.position.y - effect.source.y,
        );
        transform.rotation = Rot2::radians(pose.rotation * 0.18);
        transform.scale = pose.scale.max(Vec2::splat(0.01));
        color.0 = ACCENT.with_alpha(pose.opacity);
        if progress >= 1.0 {
            commands.entity(entity).despawn();
        }
    }
}

pub(super) fn sync_chat_messages(
    mut commands: Commands,
    mut client: Option<ResMut<ClientResource>>,
    assets: Res<UiAssets>,
    mut chat: ResMut<ChatPanelState>,
    layers: Query<(Entity, &ComputedNode, &UiGlobalTransform), With<PlayerInteractionLayer>>,
    avatars: Query<(&PlayerAvatarAnchor, &ComputedNode, &UiGlobalTransform)>,
    existing_bubbles: Query<(Entity, &ActiveChatBubble)>,
) {
    let Some(client) = client.as_deref_mut() else {
        return;
    };
    let messages = client.0.take_chat_messages();
    if messages.is_empty() {
        return;
    }
    let layer = layers.single().ok();
    let mut spawned = HashMap::<PlayerId, Entity>::new();
    for message in messages {
        let source = message.source;
        let player_name = client
            .0
            .model()
            .qigui523_game()
            .and_then(|game| {
                game.players
                    .iter()
                    .find(|player| player.id == message.source)
            })
            .map(|player| player.name.clone())
            .or_else(|| {
                client
                    .0
                    .model()
                    .texas_holdem_game()
                    .and_then(|game| {
                        game.players
                            .iter()
                            .find(|player| player.id == message.source)
                    })
                    .map(|player| player.name.clone())
            })
            .or_else(|| {
                client
                    .0
                    .model()
                    .shengji_game()
                    .and_then(|game| {
                        game.players
                            .iter()
                            .find(|player| player.id == message.source)
                    })
                    .map(|player| player.name.clone())
            })
            .unwrap_or_else(|| "玩家".to_owned());
        let text = match message.content {
            ChatContent::Text(text) => text,
            ChatContent::QuickVoice(index) => {
                let Some(text) = QUICK_VOICES.get(usize::from(index)) else {
                    continue;
                };
                if let Some(sound) = assets.quick_voice_sounds.get(usize::from(index)) {
                    commands.spawn((AudioPlayer::new(sound.clone()), PlaybackSettings::DESPAWN));
                }
                (*text).to_owned()
            }
        };
        chat.history.push_back(ChatHistoryEntry {
            player_name,
            message: text.clone(),
        });
        while chat.history.len() > CHAT_HISTORY_LIMIT {
            chat.history.pop_front();
        }

        let Some((layer, layer_node, layer_transform)) = layer else {
            continue;
        };
        let Some(anchor) =
            interaction_anchor_in_layer(source, layer_node, layer_transform, &avatars)
        else {
            continue;
        };
        for (entity, bubble) in &existing_bubbles {
            if bubble.player == source {
                commands.entity(entity).despawn();
            }
        }
        if let Some(previous) = spawned.remove(&source) {
            commands.entity(previous).despawn();
        }
        let entity = spawn_chat_bubble(
            &mut commands,
            layer,
            layer_node.size() * layer_node.inverse_scale_factor(),
            anchor,
            source,
            &text,
            &assets,
        );
        spawned.insert(source, entity);
    }
}

pub(super) fn chat_bubble_position(anchor: Vec2, layer_size: Vec2, width: f32) -> Vec2 {
    let x = if anchor.x < layer_size.x * 0.34 {
        anchor.x + 27.0
    } else if anchor.x > layer_size.x * 0.66 {
        anchor.x - width - 27.0
    } else {
        anchor.x - width * 0.5
    };
    Vec2::new(
        x.clamp(8.0, (layer_size.x - width - 8.0).max(8.0)),
        (anchor.y - 69.0).clamp(8.0, (layer_size.y - 48.0).max(8.0)),
    )
}

fn spawn_chat_bubble(
    commands: &mut Commands,
    layer: Entity,
    layer_size: Vec2,
    anchor: Vec2,
    player: PlayerId,
    message: &str,
    assets: &UiAssets,
) -> Entity {
    let character_count = message.chars().count();
    let width = (character_count as f32 * 13.0 + 26.0).clamp(76.0, 230.0);
    let position = chat_bubble_position(anchor, layer_size, width);
    let bubble = commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                left: px(position.x),
                top: px(position.y),
                width: px(width),
                min_width: px(width),
                max_width: px(width),
                min_height: px(38),
                padding: UiRect::axes(px(11), px(7)),
                align_items: AlignItems::Center,
                border: UiRect::all(px(1)),
                border_radius: BorderRadius::all(px(9)),
                ..default()
            },
            BackgroundColor(PANEL.with_alpha(0.0)),
            BorderColor::all(ACCENT.with_alpha(0.0)),
            UiTransform {
                translation: Val2::px(0.0, 9.0),
                scale: Vec2::splat(0.88),
                ..UiTransform::IDENTITY
            },
            // 高于自己的常驻得分框（1500），避免左下角气泡被遮住。
            GlobalZIndex(1600),
            FocusPolicy::Pass,
        ))
        .id();
    commands.entity(layer).add_child(bubble);
    let text = add_text(commands, bubble, message, 13.0, Color::NONE, assets);
    commands.entity(text).insert(ChatBubbleText);
    commands.entity(bubble).insert(ActiveChatBubble {
        player,
        text,
        width,
        elapsed: 0.0,
        duration: 2.8 + (character_count as f32 * 0.055).min(2.2),
    });
    bubble
}

pub(super) fn animate_chat_bubbles(
    time: Res<Time>,
    mut commands: Commands,
    layers: Query<(&ComputedNode, &UiGlobalTransform), With<PlayerInteractionLayer>>,
    avatars: Query<(&PlayerAvatarAnchor, &ComputedNode, &UiGlobalTransform)>,
    mut bubbles: Query<(
        Entity,
        &mut ActiveChatBubble,
        &mut Node,
        &mut UiTransform,
        &mut BackgroundColor,
        &mut BorderColor,
    )>,
    mut texts: Query<&mut TextColor, With<ChatBubbleText>>,
) {
    let Ok((layer_node, layer_transform)) = layers.single() else {
        return;
    };
    let layer_size = layer_node.size() * layer_node.inverse_scale_factor();
    for (entity, mut bubble, mut node, mut transform, mut background, mut border) in &mut bubbles {
        bubble.elapsed += time.delta_secs();
        if bubble.elapsed >= bubble.duration {
            commands.entity(entity).despawn();
            continue;
        }
        let Some(anchor) =
            interaction_anchor_in_layer(bubble.player, layer_node, layer_transform, &avatars)
        else {
            commands.entity(entity).despawn();
            continue;
        };
        let position = chat_bubble_position(anchor, layer_size, bubble.width);
        node.left = px(position.x);
        node.top = px(position.y);

        let enter = ease_out_cubic((bubble.elapsed / 0.18).clamp(0.0, 1.0));
        let fade = ((bubble.duration - bubble.elapsed) / 0.48).clamp(0.0, 1.0);
        let alpha = enter * fade;
        transform.translation = Val2::px(0.0, 9.0 * (1.0 - enter) - bubble.elapsed * 1.4);
        transform.scale = Vec2::splat(0.88 + enter * 0.12);
        background.0 = PANEL.with_alpha(0.96 * alpha);
        border.set_all(ACCENT.with_alpha(0.78 * alpha));
        if let Ok(mut color) = texts.get_mut(bubble.text) {
            color.0 = TEXT.with_alpha(alpha);
        }
    }
}

pub(super) fn animate_chat_panel(
    time: Res<Time>,
    assets: Res<UiAssets>,
    mut chat: ResMut<ChatPanelState>,
    mut panels: Query<&mut UiTransform, With<ChatPanel>>,
    mut icons: Query<&mut ImageNode, With<ChatToggleIcon>>,
) {
    let target = if chat.open { 0.0 } else { 1.0 };
    if (chat.slide - target).abs() < 0.001 {
        if chat.slide != target {
            chat.slide = target;
            for mut transform in &mut panels {
                transform.translation = Val2::px(CHAT_PANEL_WIDTH * target, 0.0);
            }
        }
        return;
    }
    let response = 1.0 - (-time.delta_secs() * 12.0).exp();
    chat.slide += (target - chat.slide) * response;
    if (chat.slide - target).abs() < 0.001 {
        chat.slide = target;
    }
    for mut transform in &mut panels {
        transform.translation = Val2::px(CHAT_PANEL_WIDTH * chat.slide, 0.0);
    }
    let expected = if chat.open {
        &assets.chat_close_icon
    } else {
        &assets.chat_open_icon
    };
    for mut icon in &mut icons {
        if icon.image != *expected {
            icon.image = expected.clone();
        }
    }
}

pub(super) fn animate_auto_play_robot_indicators(
    time: Res<Time>,
    mut lights: Query<(
        &AutoPlayAntennaLight,
        &mut UiTransform,
        &mut BackgroundColor,
    )>,
) {
    for (light, mut transform, mut background) in &mut lights {
        let phase = time.elapsed_secs() * 3.0 + f32::from(light.player.0) * 0.61;
        let pulse = (phase.sin() + 1.0) * 0.5;
        match light.part {
            AutoPlayAntennaLightPart::Glow => {
                transform.scale = Vec2::splat(0.72 + pulse * 0.58);
                background.0 = Color::srgba(0.32, 1.0, 0.58, 0.08 + pulse * 0.54);
            }
            AutoPlayAntennaLightPart::Ray => {
                transform.scale = Vec2::new(1.0, 0.68 + pulse * 0.42);
                background.0 = Color::srgba(0.46, 1.0, 0.68, 0.04 + pulse * 0.82);
            }
        }
    }
}

pub(super) fn sync_chat_panel_text(
    chat: Res<ChatPanelState>,
    mut history_texts: Query<&mut Text, (With<ChatHistoryText>, Without<ChatInputText>)>,
    mut input_texts: Query<(&mut Text, &mut TextColor), With<ChatInputText>>,
    mut quick_menus: Query<&mut Visibility, With<QuickVoiceMenu>>,
) {
    if !chat.is_changed() {
        return;
    }
    let history = chat
        .history
        .iter()
        .rev()
        .take(10)
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
        .map(|entry| format!("[{}(玩家)]: {}", entry.player_name, entry.message))
        .collect::<Vec<_>>()
        .join("\n");
    for mut text in &mut history_texts {
        if text.0 != history {
            text.0.clone_from(&history);
        }
    }
    let (input, color) = if chat.input.is_empty() {
        ("输入消息，回车发送".to_owned(), MUTED)
    } else {
        (chat_input_display(&chat.input), TEXT)
    };
    for (mut text, mut text_color) in &mut input_texts {
        if text.0 != input {
            text.0.clone_from(&input);
        }
        if text_color.0 != color {
            text_color.0 = color;
        }
    }
    for mut visibility in &mut quick_menus {
        let expected = if chat.open && chat.quick_voice_open {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
        if *visibility != expected {
            *visibility = expected;
        }
    }
}

pub(super) fn chat_input_display(input: &str) -> String {
    const VISIBLE_CHARS: usize = 25;
    let count = input.chars().count();
    if count <= VISIBLE_CHARS {
        return input.to_owned();
    }
    let tail = input
        .chars()
        .skip(count - VISIBLE_CHARS)
        .collect::<String>();
    format!("…{tail}")
}

pub(super) fn scroll_quick_voice_menu(
    mut wheels: MessageReader<MouseWheel>,
    mut chat: ResMut<ChatPanelState>,
    mut scrolls: Query<
        (&RelativeCursorPosition, &mut ScrollPosition, &ComputedNode),
        With<QuickVoiceScroll>,
    >,
) {
    let delta = wheels
        .read()
        .map(|wheel| match wheel.unit {
            MouseScrollUnit::Line => wheel.y * 28.0,
            MouseScrollUnit::Pixel => wheel.y,
        })
        .sum::<f32>();
    if delta == 0.0 || !chat.open || !chat.quick_voice_open {
        return;
    }
    for (cursor, mut position, node) in &mut scrolls {
        if !cursor.cursor_over() {
            continue;
        }
        let maximum =
            ((node.content_size().y - node.size().y) * node.inverse_scale_factor()).max(0.0);
        let next = (position.y - delta).clamp(0.0, maximum);
        position.y = next;
        chat.quick_voice_scroll_y = next;
    }
}

pub(super) fn accelerated_interaction_progress(progress: f32) -> f32 {
    progress.clamp(0.0, 1.0).powi(2)
}

pub(super) fn interaction_launch_sound_variant(kind: PlayerInteractionKind) -> Option<u8> {
    matches!(kind, PlayerInteractionKind::Shoe).then_some(0)
}

pub(super) fn interaction_impact_sound_variant(
    kind: PlayerInteractionKind,
    random_variant: u8,
) -> u8 {
    if matches!(kind, PlayerInteractionKind::Shoe) {
        1
    } else {
        random_variant
    }
}

pub(super) fn interaction_playback_settings() -> PlaybackSettings {
    PlaybackSettings {
        volume: Volume::Linear(INTERACTION_SOUND_VOLUME),
        ..PlaybackSettings::DESPAWN
    }
}

pub(super) fn animate_player_interactions(
    time: Res<Time>,
    assets: Res<UiAssets>,
    mut commands: Commands,
    mut effects: Query<(
        Entity,
        &mut ActivePlayerInteraction,
        &mut UiTransform,
        &mut ImageNode,
    )>,
) {
    for (entity, mut effect, mut transform, mut image) in &mut effects {
        effect.elapsed += time.delta_secs();
        if !effect.launched && effect.elapsed >= effect.appear_duration {
            if effect.play_sound
                && let Some(variant) = interaction_launch_sound_variant(effect.kind)
                && let Some(sound) = assets.interaction_sounds.get(&(effect.kind, variant))
            {
                commands.spawn((
                    AudioPlayer::new(sound.clone()),
                    interaction_playback_settings(),
                ));
            }
            effect.launched = true;
        }
        let appear = (effect.elapsed / effect.appear_duration).clamp(0.0, 1.0);
        let travel =
            ((effect.elapsed - effect.appear_duration) / effect.travel_duration).clamp(0.0, 1.0);
        let accelerated = accelerated_interaction_progress(travel);
        transform.translation = Val2::px(
            (effect.target.x - effect.source.x) * accelerated,
            (effect.target.y - effect.source.y) * accelerated,
        );
        transform.scale = Vec2::splat(0.45 + 0.55 * ease_out_cubic(appear));
        image.color = Color::WHITE.with_alpha(appear);
        transform.rotation = if interaction_rotates(effect.kind) {
            Rot2::radians(accelerated * std::f32::consts::TAU * SHOE_ROTATIONS)
        } else if matches!(
            effect.kind,
            PlayerInteractionKind::Egg | PlayerInteractionKind::Flower
        ) && travel > 0.0
            && !(effect.impacted && matches!(effect.kind, PlayerInteractionKind::Egg))
        {
            let direction = effect.target - effect.source;
            Rot2::radians(direction.y.atan2(direction.x) + std::f32::consts::FRAC_PI_2)
        } else {
            Rot2::IDENTITY
        };
        let impact_at = effect.appear_duration + effect.travel_duration;
        if !effect.impacted && effect.elapsed >= impact_at {
            if effect.play_sound {
                let variant = interaction_impact_sound_variant(effect.kind, effect.sound_variant);
                if let Some(sound) = assets.interaction_sounds.get(&(effect.kind, variant)) {
                    commands.spawn((
                        AudioPlayer::new(sound.clone()),
                        interaction_playback_settings(),
                    ));
                }
            }
            let impact_image = assets
                .interaction_images
                .get(&(effect.kind, true))
                .expect("every interaction has an impact image")
                .clone();
            if matches!(effect.kind, PlayerInteractionKind::Flower) {
                let rays = commands
                    .spawn((
                        Node {
                            position_type: PositionType::Absolute,
                            left: px(-14),
                            top: px(-14),
                            width: px(88),
                            height: px(88),
                            ..default()
                        },
                        ImageNode::new(impact_image),
                        ZIndex(2),
                        FocusPolicy::Pass,
                    ))
                    .id();
                commands.entity(entity).add_child(rays);
            } else {
                image.image = impact_image;
                if matches!(effect.kind, PlayerInteractionKind::Egg) {
                    transform.rotation = Rot2::IDENTITY;
                }
            }
            effect.impacted = true;
        }
        if effect.elapsed >= impact_at + effect.impact_duration {
            commands.entity(entity).despawn();
        }
    }
}
