use super::*;
use crate::app::presentation::{
    ACCENT, CardSize, SCORE_ROLL_DELAY, SCORE_ROLL_DURATION, TABLE_SCORE_CARD_REVEAL, add_text,
    ease_out_cubic, spawn_node,
};
use crate::app::runtime::{ClientResource, UiAssets};
use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use bevy::ui::FocusPolicy;
use leocard_client::ScoreCaptureEffect;
use leocard_protocol::PlayerId;
use std::collections::HashMap;

const SCORE_CAPTURE_TRAVEL_DURATION: f32 = 0.42;

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

pub(crate) fn sync_score_capture_effect(
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
    if (target - source_center).length() > 1.0 {
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
            .playing_cards
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

#[derive(SystemParam)]
pub(crate) struct ScoreCaptureVisuals<'w, 's> {
    cards: Query<
        'w,
        's,
        (
            Entity,
            &'static mut ActiveScoreCaptureCard,
            &'static mut UiTransform,
            &'static mut ImageNode,
        ),
    >,
    vortexes: Query<
        'w,
        's,
        (
            Entity,
            &'static mut ActiveScoreVortex,
            &'static mut BackgroundColor,
            &'static mut BorderColor,
            &'static mut UiTransform,
        ),
        Without<ActiveScoreCaptureCard>,
    >,
    gains: Query<
        'w,
        's,
        (
            Entity,
            &'static mut ActiveScoreGainText,
            &'static mut UiTransform,
        ),
        (Without<ActiveScoreCaptureCard>, Without<ActiveScoreVortex>),
    >,
}

pub(crate) fn animate_score_capture_effects(
    time: Res<Time>,
    mut commands: Commands,
    mut state: ResMut<ScoreCaptureEffectState>,
    mut visuals: ScoreCaptureVisuals,
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

    for (entity, mut card, mut transform, mut image) in &mut visuals.cards {
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

    for (entity, mut vortex, mut background, mut border, mut transform) in &mut visuals.vortexes {
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

    for (entity, mut gain, mut transform) in &mut visuals.gains {
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
pub(crate) struct VortexCardPose {
    pub position: Vec2,
    pub rotation: f32,
    pub scale: Vec2,
    pub opacity: f32,
}

pub(crate) fn vortex_card_pose(
    source: Vec2,
    target: Vec2,
    progress: f32,
    curve: f32,
) -> VortexCardPose {
    let progress = progress.clamp(0.0, 1.0);
    let position_at = |progress: f32| {
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

fn rolling_captured_score(capture: &ScoreCaptureEffect, elapsed: f32) -> u32 {
    let progress = ((elapsed - SCORE_ROLL_DELAY) / SCORE_ROLL_DURATION).clamp(0.0, 1.0);
    let eased = ease_out_cubic(progress);
    (capture.score_before as f32
        + capture.score_after.saturating_sub(capture.score_before) as f32 * eased)
        .round() as u32
}

pub(crate) fn displayed_captured_score(
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
