use super::*;
use crate::app::runtime::{ClientResource, UiAssets};
use bevy::prelude::*;
use bevy::ui::FocusPolicy;
use leocard_protocol::{PlayerId, PlayerInteractionKind};

const INTERACTION_TRAVEL_DURATION: f32 = 0.60;
const INTERACTION_IMPACT_DURATION: f32 = 1.20;
const INTERACTION_APPEAR_DURATION: f32 = 0.32;
const HEAVY_INTERACTION_TRAVEL_DURATION: f32 = 0.68;

pub(crate) fn interaction_anchor_in_layer(
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

fn interaction_target_offset(kind: PlayerInteractionKind, seed: u32) -> Vec2 {
    if matches!(kind, PlayerInteractionKind::Wine) {
        return Vec2::ZERO;
    }
    let x = (seed & 0xff) as f32 / 255.0;
    let y = ((seed >> 8) & 0xff) as f32 / 255.0;
    Vec2::new((x - 0.5) * 9.0, (y - 0.5) * 6.0)
}

pub(super) fn interaction_rotates(kind: PlayerInteractionKind) -> bool {
    matches!(kind, PlayerInteractionKind::Shoe)
}

struct InteractionProjectile {
    source: Vec2,
    target: Vec2,
    kind: PlayerInteractionKind,
    sound_variant: u8,
    play_sound: bool,
    delay: f32,
    appear_duration: f32,
    travel_duration: f32,
    impact_duration: f32,
}

impl InteractionProjectile {
    fn spawn(self, commands: &mut Commands, layer: Entity, assets: &UiAssets) {
        let effect = commands
            .spawn((
                Node {
                    position_type: PositionType::Absolute,
                    left: px(self.source.x - 30.0),
                    top: px(self.source.y - 30.0),
                    width: px(60),
                    height: px(60),
                    ..default()
                },
                ImageNode::new(
                    assets
                        .social
                        .interaction_images
                        .get(&(self.kind, false))
                        .expect("every interaction has a flight image")
                        .clone(),
                )
                .with_color(Color::WHITE.with_alpha(0.0)),
                UiTransform::default(),
                ActivePlayerInteraction {
                    source: self.source,
                    target: self.target,
                    kind: self.kind,
                    sound_variant: self.sound_variant,
                    play_sound: self.play_sound,
                    elapsed: -self.delay,
                    appear_duration: self.appear_duration,
                    travel_duration: self.travel_duration,
                    impact_duration: self.impact_duration,
                    launched: false,
                    impacted: false,
                },
                GlobalZIndex(1200),
                FocusPolicy::Pass,
            ))
            .id();
        commands.entity(layer).add_child(effect);
    }
}

fn interaction_volley_lane(index: usize) -> f32 {
    const LANES: [f32; 10] = [-2.0, 2.0, -1.0, 1.0, 0.0, -1.5, 1.5, -0.5, 0.5, 0.0];
    LANES[index] * 9.0
}

fn interaction_volley_kind(kind: PlayerInteractionKind) -> Option<PlayerInteractionKind> {
    match kind {
        PlayerInteractionKind::Wine => Some(PlayerInteractionKind::Flower),
        PlayerInteractionKind::Shoe => Some(PlayerInteractionKind::Egg),
        PlayerInteractionKind::Flower | PlayerInteractionKind::Egg => None,
    }
}

fn interaction_volley_interval(kind: PlayerInteractionKind) -> f32 {
    match kind {
        PlayerInteractionKind::Egg => 0.09,
        PlayerInteractionKind::Flower => 0.055,
        PlayerInteractionKind::Wine | PlayerInteractionKind::Shoe => 0.0,
    }
}

pub(crate) fn sync_player_interactions(
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
                InteractionProjectile {
                    source: source + perpendicular * lane,
                    target: target + perpendicular * (lane * 0.72),
                    kind: volley_kind,
                    sound_variant: ((interaction.seed >> (index % 16)) & 1) as u8,
                    play_sound: !matches!(interaction.kind, PlayerInteractionKind::Wine)
                        || index == 0,
                    delay: index as f32 * volley_interval,
                    appear_duration: 0.07,
                    travel_duration: 0.40,
                    impact_duration: 0.42,
                }
                .spawn(&mut commands, layer, &assets);
            }
            InteractionProjectile {
                source,
                target,
                kind: interaction.kind,
                sound_variant,
                play_sound: true,
                delay: 9.0 * volley_interval + 0.55,
                appear_duration: 0.16,
                travel_duration: HEAVY_INTERACTION_TRAVEL_DURATION,
                impact_duration: INTERACTION_IMPACT_DURATION,
            }
            .spawn(&mut commands, layer, &assets);
        } else {
            InteractionProjectile {
                source,
                target,
                kind: interaction.kind,
                sound_variant,
                play_sound: true,
                delay: 0.0,
                appear_duration: INTERACTION_APPEAR_DURATION,
                travel_duration: INTERACTION_TRAVEL_DURATION,
                impact_duration: INTERACTION_IMPACT_DURATION,
            }
            .spawn(&mut commands, layer, &assets);
        }
    }
}
