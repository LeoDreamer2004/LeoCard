//! 玩家互动投射物的运动、声音与命中反馈。

use super::{ActivePlayerInteraction, interaction_rotates};
use crate::app::presentation::ease_out_cubic;
use crate::app::runtime::UiAssets;
use bevy::audio::Volume;
use bevy::prelude::*;
use bevy::ui::FocusPolicy;
use leocard_protocol::PlayerInteractionKind;

const SHOE_ROTATIONS: f32 = 2.0;
const INTERACTION_SOUND_VOLUME: f32 = 0.5;

fn accelerated_interaction_progress(progress: f32) -> f32 {
    progress.clamp(0.0, 1.0).powi(2)
}

fn interaction_launch_sound_variant(kind: PlayerInteractionKind) -> Option<u8> {
    matches!(kind, PlayerInteractionKind::Shoe).then_some(0)
}

fn interaction_impact_sound_variant(kind: PlayerInteractionKind, random_variant: u8) -> u8 {
    if matches!(kind, PlayerInteractionKind::Shoe) {
        1
    } else {
        random_variant
    }
}

fn interaction_playback_settings() -> PlaybackSettings {
    PlaybackSettings {
        volume: Volume::Linear(INTERACTION_SOUND_VOLUME),
        ..PlaybackSettings::DESPAWN
    }
}

pub(crate) fn animate_player_interactions(
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
                && let Some(sound) = assets.audio.interaction_sounds.get(&(effect.kind, variant))
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
                if let Some(sound) = assets.audio.interaction_sounds.get(&(effect.kind, variant)) {
                    commands.spawn((
                        AudioPlayer::new(sound.clone()),
                        interaction_playback_settings(),
                    ));
                }
            }
            let impact_image = assets
                .social
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
