//! 升级收分与终局结算演出。

use super::*;

const SHENGJI_KITTY_CARD_INTERVAL: f32 = 0.075;
const SHENGJI_KITTY_CARD_ENTRY_DURATION: f32 = 0.22;
const SHENGJI_KITTY_MULTIPLIER_DELAY: f32 = 1.12;
const SHENGJI_KITTY_MULTIPLIER_DURATION: f32 = 0.62;
const SHENGJI_TOTAL_ABSORB_DELAY: f32 = 2.24;
const SHENGJI_TOTAL_ABSORB_DURATION: f32 = 0.78;

pub fn sync_shengji_score_capture_effect(
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

pub fn animate_shengji_score_capture_score(
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

fn rolling_shengji_captured_score(capture: &ShengjiScoreCaptureEffect, elapsed: f32) -> u32 {
    let progress = ((elapsed - SCORE_ROLL_DELAY) / SCORE_ROLL_DURATION).clamp(0.0, 1.0);
    let eased = ease_out_cubic(progress);
    (capture.score_before as f32
        + capture.score_after.saturating_sub(capture.score_before) as f32 * eased)
        .round() as u32
}

pub fn displayed_shengji_captured_score(
    state: &ShengjiScoreCaptureEffectState,
    fallback: u32,
) -> u32 {
    state.active.as_ref().map_or(fallback, |active| {
        rolling_shengji_captured_score(&active.capture, active.elapsed)
    })
}

pub fn update_shengji_settlement_animation(
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
            assets.audio.summary_score_sound.clone()
        } else {
            assets.audio.summary_die_sound.clone()
        };
        commands.spawn((AudioPlayer::new(sound), PlaybackSettings::DESPAWN));
        animation.outcome_sound_played = true;
    }
}

pub fn animate_shengji_settlement_visuals(
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

pub fn spawn_shengji_settlement_absorption(
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

pub fn animate_shengji_score_absorbs(
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
