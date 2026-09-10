use super::{
    ShengjiBottomFlipPanelElement, ShengjiBottomFlipVisual, ShengjiBottomFlipVisualKind,
    ShengjiDealerBadge, ShengjiLevelIndicator, ShengjiPowerOutageVisual,
    ShengjiPowerOutageVisualKind, ShengjiPresentationDivider, ShengjiPresentationKind,
    ShengjiPresentationPacket, ShengjiPresentationRoot, ShengjiPresentationState,
    ShengjiPresentationText, ShengjiPresentationVeil, ShengjiSoundAssets, ShengjiTrumpKillVisual,
    ShengjiTrumpKillVisualKind, presentation_color,
};
use crate::app::presentation::{ACCENT, HEADER_BG, ease_out_cubic};
use crate::app::shell::{PlayerAvatarAnchor, UiState};
use bevy::audio::Volume;
use bevy::prelude::*;

pub(crate) fn advance_shengji_presentation(
    time: Res<Time>,
    mut state: ResMut<ShengjiPresentationState>,
    mut ui: ResMut<UiState>,
) {
    let delta = time.delta_secs();
    if state.previous_trick_reveal_remaining > 0.0 {
        state.previous_trick_reveal_remaining =
            (state.previous_trick_reveal_remaining - delta).max(0.0);
        if state.previous_trick_reveal_remaining == 0.0 {
            ui.dirty = true;
        }
    }
    let Some(active) = state.active.as_mut() else {
        return;
    };
    active.elapsed += delta;
    if active.elapsed >= active.duration {
        state.active = state.queued.pop_front();
        ui.dirty = true;
    }
}

pub(crate) fn play_shengji_audio_cues(
    time: Res<Time>,
    assets: Res<ShengjiSoundAssets>,
    mut state: ResMut<ShengjiPresentationState>,
    mut commands: Commands,
) {
    let mut waiting = Vec::with_capacity(state.audio_cues.len());
    let mut ready = Vec::new();
    for mut cue in std::mem::take(&mut state.audio_cues) {
        cue.remaining -= time.delta_secs();
        if cue.remaining <= 0.0 {
            ready.push(cue);
        } else {
            waiting.push(cue);
        }
    }
    state.audio_cues = waiting;
    for cue in ready {
        let variants = assets.variants(cue.kind);
        if variants.is_empty() {
            continue;
        }
        commands.spawn((
            AudioPlayer::new(variants[cue.seed as usize % variants.len()].clone()),
            PlaybackSettings {
                volume: Volume::Linear(cue.volume),
                ..PlaybackSettings::DESPAWN
            },
        ));
    }
}

#[expect(
    clippy::type_complexity,
    clippy::too_many_arguments,
    reason = "independent Bevy queries model the presentation's mutually exclusive layers"
)]
pub(crate) fn animate_shengji_presentation(
    state: Res<ShengjiPresentationState>,
    mut roots: Query<
        (&ShengjiPresentationRoot, &mut UiTransform, &mut Visibility),
        (
            With<ShengjiPresentationRoot>,
            Without<ShengjiPresentationVeil>,
            Without<ShengjiPresentationDivider>,
            Without<ShengjiPowerOutageVisual>,
            Without<ShengjiTrumpKillVisual>,
            Without<ShengjiBottomFlipVisual>,
        ),
    >,
    mut veils: Query<
        &mut BackgroundColor,
        (
            With<ShengjiPresentationVeil>,
            Without<ShengjiPowerOutageVisual>,
            Without<ShengjiTrumpKillVisual>,
            Without<ShengjiBottomFlipVisual>,
        ),
    >,
    mut texts: Query<(&ShengjiPresentationText, &mut TextColor)>,
    mut dividers: Query<
        (&mut UiTransform, &mut BackgroundColor),
        (
            Without<ShengjiPresentationRoot>,
            Without<ShengjiPresentationVeil>,
            Without<ShengjiPresentationPacket>,
            Without<ShengjiPowerOutageVisual>,
            Without<ShengjiTrumpKillVisual>,
            Without<ShengjiBottomFlipVisual>,
            With<ShengjiPresentationDivider>,
        ),
    >,
    mut packets: Query<
        (
            &ShengjiPresentationPacket,
            &mut Node,
            &mut UiTransform,
            &mut ImageNode,
        ),
        (
            Without<ShengjiPresentationRoot>,
            Without<ShengjiPresentationDivider>,
            Without<ShengjiPowerOutageVisual>,
            Without<ShengjiTrumpKillVisual>,
            Without<ShengjiBottomFlipVisual>,
        ),
    >,
    mut power_visuals: Query<
        (
            &ShengjiPowerOutageVisual,
            &mut Node,
            &mut UiTransform,
            &mut BackgroundColor,
            Option<&mut BorderColor>,
            &mut Visibility,
        ),
        (
            Without<ShengjiPresentationRoot>,
            Without<ShengjiPresentationVeil>,
            Without<ShengjiPresentationDivider>,
            Without<ShengjiPresentationPacket>,
            Without<ShengjiTrumpKillVisual>,
            Without<ShengjiBottomFlipVisual>,
        ),
    >,
    mut trump_kill_visuals: Query<
        (
            &ShengjiTrumpKillVisual,
            &mut Node,
            &mut UiTransform,
            Option<&mut ImageNode>,
            &mut BackgroundColor,
            Option<&mut BorderColor>,
            &mut Visibility,
        ),
        (
            Without<ShengjiPresentationRoot>,
            Without<ShengjiPresentationVeil>,
            Without<ShengjiPresentationDivider>,
            Without<ShengjiPresentationPacket>,
            Without<ShengjiPowerOutageVisual>,
            Without<ShengjiBottomFlipVisual>,
        ),
    >,
    mut bottom_flip_visuals: Query<
        (
            &ShengjiBottomFlipVisual,
            &mut Node,
            &mut UiTransform,
            &mut BackgroundColor,
            &mut Visibility,
        ),
        (
            Without<ShengjiPresentationRoot>,
            Without<ShengjiPresentationVeil>,
            Without<ShengjiPresentationDivider>,
            Without<ShengjiPresentationPacket>,
            Without<ShengjiPowerOutageVisual>,
            Without<ShengjiTrumpKillVisual>,
        ),
    >,
) {
    let Some(active) = state.active.as_ref() else {
        for (_, _, mut visibility) in &mut roots {
            *visibility = Visibility::Hidden;
        }
        return;
    };
    let progress = (active.elapsed / active.duration).clamp(0.0, 1.0);
    let enter = ease_out_cubic((progress / 0.16).clamp(0.0, 1.0));
    let exit = ease_out_cubic(((progress - 0.82) / 0.18).clamp(0.0, 1.0));
    let alpha = enter * (1.0 - exit);
    for (root, mut transform, mut visibility) in &mut roots {
        *visibility = Visibility::Visible;
        transform.scale = Vec2::splat(0.98 + 0.02 * enter);
        transform.translation = Val2::px(
            root.base_translation.x,
            root.base_translation.y + 6.0 * (1.0 - enter) - 3.0 * exit,
        );
    }
    let outage = matches!(active.kind, ShengjiPresentationKind::PowerOutage { .. });
    for mut background in &mut veils {
        let outage_alpha = if !outage {
            0.0
        } else if progress < 0.10 {
            ease_out_cubic(progress / 0.10)
        } else if progress < 0.55 {
            1.0
        } else {
            1.0 - ease_out_cubic(((progress - 0.55) / 0.18).clamp(0.0, 1.0))
        };
        background.0 = Color::BLACK.with_alpha(0.46 * outage_alpha);
    }
    for (text, mut color) in &mut texts {
        color.0 = text.base_color.with_alpha(alpha);
    }
    for (mut transform, mut background) in &mut dividers {
        transform.scale = Vec2::new(0.72 + 0.28 * enter, 1.0);
        background.0 = presentation_color(&active.kind).with_alpha(0.72 * alpha);
    }
    for (packet, mut node, mut transform, mut image) in &mut packets {
        let spread = packet.index as f32 - (packet.count.saturating_sub(1) as f32 * 0.5);
        let stagger = packet.index as f32 / packet.count.max(1) as f32 * 0.08
            + packet.route_index as f32 * 0.015;
        let packet_progress = ((progress - 0.06 - stagger) / 0.68).clamp(0.0, 1.0);
        let travel = ease_out_cubic(packet_progress);
        let position = packet.start.lerp(packet.end, travel);
        node.left = percent(position.x);
        node.top = percent(position.y);

        let direction = (packet.end - packet.start).normalize_or_zero();
        let perpendicular = Vec2::new(-direction.y, direction.x);
        let arc = (packet_progress * std::f32::consts::PI).sin();
        let offset = perpendicular * (spread * 7.0 + arc * (24.0 + packet.index as f32));
        transform.translation = Val2::px(-17.0 + offset.x, -24.0 + offset.y);
        transform.rotation = Rot2::radians((spread * 1.2_f32).to_radians());
        transform.scale = Vec2::splat(0.92 + arc * 0.08);
        image.color = Color::WHITE.with_alpha(alpha);
    }
    for (visual, mut node, mut transform, mut background, border, mut visibility) in
        &mut power_visuals
    {
        match visual.kind {
            ShengjiPowerOutageVisualKind::Spark {
                index,
                count,
                start,
                end,
            } => {
                let stagger = index as f32 / count.max(1) as f32 * 0.12;
                let local = ((progress - 0.15 - stagger) / 0.27).clamp(0.0, 1.0);
                let alive = progress >= 0.15 + stagger && local < 1.0;
                *visibility = if alive {
                    Visibility::Visible
                } else {
                    Visibility::Hidden
                };
                let travel = ease_out_cubic(local);
                let direction = (end - start).normalize_or_zero();
                let perpendicular = Vec2::new(-direction.y, direction.x);
                let polarity = if index % 2 == 0 { 1.0 } else { -1.0 };
                let position = start.lerp(end, travel)
                    + perpendicular * ((local * std::f32::consts::PI).sin() * polarity * 1.8);
                node.left = percent(position.x);
                node.top = percent(position.y);
                transform.translation = Val2::px(-4.0, -4.0);
                transform.scale =
                    Vec2::splat(0.72 + (local * std::f32::consts::PI).sin().max(0.0) * 0.58);
                let spark_alpha = (local * std::f32::consts::PI).sin().max(0.0);
                background.0 = Color::srgba(0.42, 0.88, 1.0, spark_alpha * 0.96);
            }
            ShengjiPowerOutageVisualKind::TargetRing { target } => {
                let local = ((progress - 0.38) / 0.30).clamp(0.0, 1.0);
                let alive = (0.38..0.68).contains(&progress);
                *visibility = if alive {
                    Visibility::Visible
                } else {
                    Visibility::Hidden
                };
                node.left = percent(target.x);
                node.top = percent(target.y);
                transform.translation = Val2::px(-25.0, -25.0);
                transform.scale = Vec2::splat(0.54 + ease_out_cubic(local) * 0.82);
                background.0 = Color::NONE;
                if let Some(mut border) = border {
                    let ring_alpha = (local * std::f32::consts::PI).sin().max(0.0);
                    border.set_all(Color::srgba(0.42, 0.88, 1.0, ring_alpha * 0.92));
                }
            }
            ShengjiPowerOutageVisualKind::LevelFlash { target } => {
                let local = ((progress - 0.45) / 0.43).clamp(0.0, 1.0);
                let enter = ease_out_cubic((local / 0.20).clamp(0.0, 1.0));
                let exit = ease_out_cubic(((local - 0.76) / 0.24).clamp(0.0, 1.0));
                let flash_alpha = enter * (1.0 - exit);
                *visibility = if flash_alpha > 0.0 {
                    Visibility::Visible
                } else {
                    Visibility::Hidden
                };
                node.left = percent(target.x);
                node.top = percent(target.y);
                transform.translation = Val2::px(-58.0, -17.0 + 4.0 * (1.0 - enter));
                transform.scale = Vec2::new(0.82 + 0.18 * enter, 1.0);
                background.0 = HEADER_BG.with_alpha(flash_alpha * 0.92);
                if let Some(mut border) = border {
                    border.set_all(ACCENT.with_alpha(flash_alpha * 0.88));
                }
            }
        }
    }
    for (visual, mut node, mut transform, mut background, mut visibility) in
        &mut bottom_flip_visuals
    {
        match visual.kind {
            ShengjiBottomFlipVisualKind::ScanSpark {
                player_index,
                player_count,
                spark_index,
                spark_count,
                start,
                end,
            } => {
                let player_stagger = player_index as f32 / player_count.max(1) as f32 * 0.10;
                let spark_stagger = spark_index as f32 / spark_count.max(1) as f32 * 0.055;
                let local =
                    ((progress - 0.10 - player_stagger - spark_stagger) / 0.22).clamp(0.0, 1.0);
                let alive = progress >= 0.10 + player_stagger + spark_stagger && local < 1.0;
                *visibility = if alive {
                    Visibility::Visible
                } else {
                    Visibility::Hidden
                };
                let travel = ease_out_cubic(local);
                let direction = (end - start).normalize_or_zero();
                let perpendicular = Vec2::new(-direction.y, direction.x);
                let polarity = if spark_index % 2 == 0 { 1.0 } else { -1.0 };
                let position = start.lerp(end, travel);
                node.left = percent(position.x);
                node.top = percent(position.y);
                let arc = (local * std::f32::consts::PI).sin();
                transform.translation = Val2::px(
                    -3.5 + perpendicular.x * arc * polarity * 4.0,
                    -3.5 + perpendicular.y * arc * polarity * 4.0,
                );
                transform.scale = Vec2::splat(0.70 + arc * 0.52);
                background.0 = Color::srgba(0.44, 0.90, 1.0, arc * 0.92);
            }
            ShengjiBottomFlipVisualKind::ReplySpark {
                player_index,
                player_count,
                spark_index,
                spark_count,
                start,
                end,
            } => {
                let player_stagger = player_index as f32 / player_count.max(1) as f32 * 0.10;
                let spark_stagger = spark_index as f32 / spark_count.max(1) as f32 * 0.05;
                let local =
                    ((progress - 0.35 - player_stagger - spark_stagger) / 0.24).clamp(0.0, 1.0);
                let alive = progress >= 0.35 + player_stagger + spark_stagger && local < 1.0;
                *visibility = if alive {
                    Visibility::Visible
                } else {
                    Visibility::Hidden
                };
                let travel = ease_out_cubic(local);
                let direction = (end - start).normalize_or_zero();
                let perpendicular = Vec2::new(-direction.y, direction.x);
                let polarity = if spark_index % 2 == 0 { 1.0 } else { -1.0 };
                let position = start.lerp(end, travel);
                node.left = percent(position.x);
                node.top = percent(position.y);
                let arc = (local * std::f32::consts::PI).sin();
                transform.translation = Val2::px(
                    -4.0 + perpendicular.x * arc * polarity * 5.0,
                    -4.0 + perpendicular.y * arc * polarity * 5.0,
                );
                transform.rotation = Rot2::radians(polarity * arc * 0.35);
                transform.scale = Vec2::new(0.78 + arc * 0.38, 0.78 + arc * 0.38);
                background.0 = Color::srgba(0.40, 0.94, 0.62, arc * 0.94);
            }
        }
    }
    for (visual, mut node, mut transform, image, mut background, border, mut visibility) in
        &mut trump_kill_visuals
    {
        match visual.kind {
            ShengjiTrumpKillVisualKind::Target => {
                let reveal = ease_out_cubic((progress / 0.16).clamp(0.0, 1.0));
                let fade = 1.0 - ease_out_cubic(((progress - 0.84) / 0.16).clamp(0.0, 1.0));
                let impact = ((progress - 0.48) / 0.16).clamp(0.0, 1.0);
                let pulse = (impact * std::f32::consts::PI).sin().max(0.0);
                *visibility = if fade > 0.0 {
                    Visibility::Visible
                } else {
                    Visibility::Hidden
                };
                transform.scale = Vec2::splat((0.68 + 0.32 * reveal) * (1.0 + pulse * 0.16));
                if let Some(mut image) = image {
                    image.color = Color::srgba(0.36, 0.94, 0.67, reveal * fade * 0.92);
                }
                background.0 = Color::NONE;
            }
            ShengjiTrumpKillVisualKind::Dart => {
                let local = ((progress - 0.10) / 0.42).clamp(0.0, 1.0);
                let alive = (0.10..0.58).contains(&progress);
                *visibility = if alive {
                    Visibility::Visible
                } else {
                    Visibility::Hidden
                };
                let travel = ease_out_cubic(local);
                node.left = px(-42.0 + 118.0 * travel);
                node.top = px(4.0 - (local * std::f32::consts::PI).sin() * 7.0);
                transform.rotation = Rot2::radians((-45.0 + (1.0 - local) * 3.0).to_radians());
                transform.scale = Vec2::splat(0.88 + 0.12 * travel);
                if let Some(mut image) = image {
                    let fade = 1.0 - ((local - 0.90) / 0.10).clamp(0.0, 1.0);
                    image.color = Color::srgba(1.0, 0.78, 0.16, fade);
                }
                background.0 = Color::NONE;
            }
            ShengjiTrumpKillVisualKind::ImpactRing => {
                let local = ((progress - 0.48) / 0.30).clamp(0.0, 1.0);
                let alive = (0.48..0.78).contains(&progress);
                *visibility = if alive {
                    Visibility::Visible
                } else {
                    Visibility::Hidden
                };
                transform.scale = Vec2::splat(0.42 + ease_out_cubic(local) * 1.04);
                background.0 = Color::NONE;
                if let Some(mut border) = border {
                    border.set_all(ACCENT.with_alpha((1.0 - local) * 0.90));
                }
            }
        }
    }
}

pub(crate) fn animate_shengji_bottom_flip_markers(
    state: Res<ShengjiPresentationState>,
    mut panel_elements: Query<
        (
            &ShengjiBottomFlipPanelElement,
            &mut UiTransform,
            &mut Visibility,
        ),
        Without<PlayerAvatarAnchor>,
    >,
    mut avatars: Query<
        (&PlayerAvatarAnchor, &mut UiTransform),
        Without<ShengjiBottomFlipPanelElement>,
    >,
) {
    let bottom_flip = state.active.as_ref().and_then(|active| {
        let ShengjiPresentationKind::BottomFlip {
            matches, dealer, ..
        } = &active.kind
        else {
            return None;
        };
        Some((
            (active.elapsed / active.duration).clamp(0.0, 1.0),
            matches.as_slice(),
            *dealer,
        ))
    });

    let Some((progress, matches, dealer)) = bottom_flip else {
        for (_, mut transform, mut visibility) in &mut panel_elements {
            *transform = UiTransform::IDENTITY;
            *visibility = Visibility::Visible;
        }
        for (_, mut transform) in &mut avatars {
            *transform = UiTransform::IDENTITY;
        }
        return;
    };

    for (element, mut transform, mut visibility) in &mut panel_elements {
        match *element {
            ShengjiBottomFlipPanelElement::CentralCard => {
                let reveal = ease_out_cubic((progress / 0.16).clamp(0.0, 1.0));
                *visibility = Visibility::Visible;
                transform.scale = Vec2::new(0.06 + reveal * 0.94, 0.90 + reveal * 0.10);
                transform.translation = Val2::px(0.0, 5.0 * (1.0 - reveal));
            }
            ShengjiBottomFlipPanelElement::MatchRow {
                player,
                index,
                count,
            } => {
                let still_matches = matches.iter().any(|matched| matched.player == player);
                let stagger = index as f32 / count.max(1) as f32 * 0.10;
                let reveal = ease_out_cubic(((progress - 0.50 - stagger) / 0.13).clamp(0.0, 1.0));
                *visibility = if still_matches && reveal > 0.0 {
                    Visibility::Visible
                } else {
                    Visibility::Hidden
                };
                let direction = if index % 2 == 0 { -1.0 } else { 1.0 };
                transform.translation = Val2::px(direction * 18.0 * (1.0 - reveal), 0.0);
                transform.scale = Vec2::new(0.94 + 0.06 * reveal, 0.94 + 0.06 * reveal);
            }
            ShengjiBottomFlipPanelElement::DealerLine => {
                let reveal = ease_out_cubic(((progress - 0.68) / 0.13).clamp(0.0, 1.0));
                *visibility = if reveal > 0.0 {
                    Visibility::Visible
                } else {
                    Visibility::Hidden
                };
                let bounce = (reveal * std::f32::consts::PI).sin().max(0.0);
                transform.translation = Val2::px(0.0, 7.0 * (1.0 - reveal));
                transform.scale = Vec2::splat(0.88 + reveal * 0.12 + bounce * 0.08);
            }
        }
    }

    for (anchor, mut transform) in &mut avatars {
        *transform = UiTransform::IDENTITY;
        let Some(index) = matches
            .iter()
            .position(|matched| matched.player == anchor.0)
        else {
            continue;
        };
        let stagger = index as f32 / matches.len().max(1) as f32 * 0.10;
        let response = ((progress - 0.27 - stagger) / 0.22).clamp(0.0, 1.0);
        let response_pulse = (response * std::f32::consts::PI).sin().max(0.0);
        let dealer_pulse = if dealer == Some(anchor.0) {
            (((progress - 0.63) / 0.20).clamp(0.0, 1.0) * std::f32::consts::PI)
                .sin()
                .max(0.0)
        } else {
            0.0
        };
        transform.scale = Vec2::splat(1.0 + response_pulse * 0.14 + dealer_pulse * 0.16);
        transform.rotation = Rot2::radians(response_pulse * 0.035 - dealer_pulse * 0.025);
    }
}

pub(crate) fn animate_shengji_power_outage_markers(
    state: Res<ShengjiPresentationState>,
    mut badges: Query<
        (&mut UiTransform, &mut BackgroundColor, &mut Visibility),
        With<ShengjiDealerBadge>,
    >,
    mut levels: Query<
        (&ShengjiLevelIndicator, &mut UiTransform, &mut TextColor),
        Without<ShengjiDealerBadge>,
    >,
) {
    let progress = state.active.as_ref().and_then(|active| {
        matches!(active.kind, ShengjiPresentationKind::PowerOutage { .. })
            .then_some((active.elapsed / active.duration).clamp(0.0, 1.0))
    });
    let Some(progress) = progress else {
        for (mut transform, mut background, mut visibility) in &mut badges {
            *visibility = Visibility::Visible;
            *transform = UiTransform::IDENTITY;
            background.0 = ACCENT;
        }
        for (indicator, mut transform, mut color) in &mut levels {
            *transform = UiTransform::IDENTITY;
            color.0 = indicator.base_color;
        }
        return;
    };

    let badge_reveal = ease_out_cubic(((progress - 0.39) / 0.14).clamp(0.0, 1.0));
    for (mut transform, mut background, mut visibility) in &mut badges {
        *visibility = if badge_reveal > 0.0 {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
        transform.scale = Vec2::splat(0.52 + 0.58 * badge_reveal - 0.10 * badge_reveal.powi(2));
        background.0 = ACCENT.with_alpha(badge_reveal);
    }

    let dim = 1.0 - 0.82 * ease_out_cubic((progress / 0.10).clamp(0.0, 1.0));
    let level_reveal = ease_out_cubic(((progress - 0.43) / 0.18).clamp(0.0, 1.0));
    for (indicator, mut transform, mut color) in &mut levels {
        transform.scale = Vec2::new(0.08 + 0.92 * level_reveal, 1.0);
        transform.translation = Val2::px(0.0, -3.0 * (1.0 - level_reveal));
        color.0 = indicator
            .base_color
            .with_alpha(dim.max(0.18 + level_reveal * 0.82));
    }
}
