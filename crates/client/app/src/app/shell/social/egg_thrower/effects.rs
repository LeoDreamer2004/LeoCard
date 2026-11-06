use super::super::interaction::{InteractionProjectile, interaction_volley_lane};
use super::choreography::EggBarrageTimeline;
use crate::app::runtime::{ClientResource, UiAssets};
use bevy::prelude::*;
use leocard_client::NetworkState;
use leocard_protocol::{PlayerInteraction, PlayerInteractionKind, ProfileId, RoomId};
use leocard_ui::UiComposer;
use std::time::Duration;

#[derive(Component)]
pub(crate) struct EggBarrageAnimation {
    interaction: PlayerInteraction,
    room: RoomId,
    target_profile: ProfileId,
    path: (Vec2, Vec2),
    timeline: EggBarrageTimeline,
}

pub(crate) fn add_egg_barrage(
    commands: &mut UiComposer,
    layer: Entity,
    interaction: PlayerInteraction,
    target_profile: ProfileId,
    path: (Vec2, Vec2),
    room: RoomId,
    now: Duration,
) {
    // Only this small scheduler exists up front. Projectiles are created on their beat.
    commands.effect_child(
        layer,
        EggBarrageAnimation {
            interaction,
            room,
            target_profile,
            path,
            timeline: EggBarrageTimeline::new(now, interaction.seed),
        },
    );
}

pub(crate) fn animate_egg_barrages(
    time: Res<Time<Real>>,
    client: Option<Res<ClientResource>>,
    assets: Res<UiAssets>,
    mut commands: UiComposer,
    mut barrages: Query<(Entity, &ChildOf, &mut EggBarrageAnimation)>,
) {
    for (entity, parent, mut barrage) in &mut barrages {
        let valid = client.as_deref().is_some_and(|client| {
            let model = client.0.model();
            matches!(client.0.state(), NetworkState::Connected(_))
                && model.room_id() == barrage.room
                && model
                    .player(barrage.interaction.target)
                    .is_some_and(|player| player.profile_id == barrage.target_profile)
        });
        if !valid {
            commands.entity(entity).despawn();
            continue;
        }
        let now = time.elapsed();
        if !barrage.timeline.ready(now) {
            continue;
        }
        let beat = barrage.timeline.next_beat();
        let kind = beat.kind;
        let finisher = kind == PlayerInteractionKind::Shoe;
        let index = usize::from(barrage.timeline.emitted);
        let seed = barrage.interaction.seed.wrapping_add(index as u32 * 7919);
        let (source, target) = barrage.path;
        let direction = (target - source).normalize_or_zero();
        let perpendicular = Vec2::new(-direction.y, direction.x);
        let lane = if finisher {
            0.0
        } else {
            interaction_volley_lane(index) * 1.6
        };
        InteractionProjectile {
            source: source + perpendicular * lane,
            target: target + perpendicular * lane * 0.72,
            kind,
            sound_variant: (seed & 1) as u8,
            play_sound: true,
            sound_gain: beat.gain,
            delay: 0.0,
            appear_duration: if finisher { 0.32 } else { 0.07 },
            travel_duration: if finisher { 0.78 } else { 0.40 },
            impact_duration: if finisher { 1.2 } else { 0.48 },
            size: if finisher {
                144.0
            } else {
                60.0 * (0.84 + 0.16 * beat.gain)
            },
        }
        .spawn(&mut commands, parent.parent(), &assets);
        barrage.timeline.advance(now);
        if barrage.timeline.finished() {
            commands.entity(entity).despawn();
        }
    }
}
