use super::MahjongAssets;
use super::layout::MahjongLayout;
use crate::app::presentation::mahjong::{
    CAMERA_POSITION, MahjongProjectionFrame, MahjongTableRoot, REFERENCE_DESIGN_HEIGHT,
    REFERENCE_SIZE, ground_from_layout, project_reference, reference_ray,
};
use crate::app::presentation::{DESIGN_WIDTH, StartGameSeatTransition, ease_out_cubic};
use crate::app::runtime::{ClientResource, ClientUpdateSet};
use bevy::audio::Volume;
use bevy::picking::Pickable;
use bevy::prelude::*;
use bevy::render::render_resource::AsBindGroup;
use bevy::shader::ShaderRef;
use leocard_client::ClientPhaseRef;
use leocard_protocol::{GameSnapshot, MahjongPhaseView, MahjongSnapshot, MatchId};
use std::f32::consts::{FRAC_PI_2, PI, TAU};

const ROLL_DURATION: f32 = 0.90;
const HOLD_DURATION: f32 = 1.0;
const ROLL_HOPS: f32 = 3.0;
const FIRST_LANDING: f32 = ROLL_DURATION / ROLL_HOPS;
// Retain the roll briefly through dealing so a hidden roll cannot be spawned again.
const LAST_ROLL_BATCH: u8 = 5;
const REFERENCE_SCALE: f32 = DESIGN_WIDTH / REFERENCE_SIZE.x;
const DIE_HALF_EDGE: f32 = if cfg!(target_os = "android") {
    0.40
} else {
    0.34
};

pub(super) struct MahjongDicePlugin;

impl Plugin for MahjongDicePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(UiMaterialPlugin::<MahjongDieMaterial>::default())
            .add_systems(
                Update,
                (sync_mahjong_dice, animate_mahjong_dice)
                    .chain()
                    .in_set(ClientUpdateSet::Animate),
            );
    }
}

#[derive(Asset, TypePath, AsBindGroup, Clone)]
struct MahjongDieMaterial {
    #[uniform(0)]
    ray_origin: Vec4,
    #[uniform(0)]
    ray_center: Vec4,
    #[uniform(0)]
    ray_right: Vec4,
    #[uniform(0)]
    ray_down: Vec4,
    #[uniform(0)]
    light: Vec4,
    #[uniform(0)]
    shadow: Vec4,
    #[uniform(0)]
    red: u32,
}

impl UiMaterial for MahjongDieMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/mahjong_die.wesl".into()
    }
}

#[derive(Component)]
struct MahjongDiceRoll {
    hand: (MatchId, u8),
    elapsed: f32,
}

#[derive(Component)]
struct MahjongRollingDie {
    source: Vec2,
    target: Vec2,
    rest: Quat,
    lane: f32,
    red: bool,
}

impl MahjongRollingDie {
    fn projection(&self, progress: f32) -> (Rect, MahjongDieMaterial) {
        let movement = ease_out_cubic(progress);
        let travel = self.source.lerp(self.target, movement);
        let ground = Vec3::new(travel.x, 0.0, travel.y);
        let remaining = 1.0 - movement;
        let yaw = Quat::from_rotation_y(self.lane * (0.20 + TAU * remaining));
        let tumble = Quat::from_rotation_x(1.5 * TAU * remaining)
            * Quat::from_rotation_z(self.lane * TAU * remaining);
        let orientation = yaw * tumble * self.rest;
        let inverse = orientation.inverse();
        // Support the lowest corner on the felt throughout the roll, then add the bounce.
        let up = (inverse * Vec3::Y).abs().element_sum() * DIE_HALF_EDGE;
        let hop = (progress * PI * ROLL_HOPS).sin().abs() * (1.0 - progress) * 1.10;
        let center = ground + Vec3::Y * (up + hop);
        let shadow_center = project_reference(ground + Vec3::new(0.06, 0.0, 0.08));
        let shadow_width = (project_reference(ground + Vec3::X * DIE_HALF_EDGE).x
            - project_reference(ground).x)
            .abs();
        let shadow_radius = Vec2::new(1.55, 1.05) * shadow_width;
        let mut min = shadow_center - shadow_radius;
        let mut max = shadow_center + shadow_radius;
        for x in [-1.0, 1.0] {
            for y in [-1.0, 1.0] {
                for z in [-1.0, 1.0] {
                    let vertex = center + orientation * (Vec3::new(x, y, z) * DIE_HALF_EDGE);
                    let pixel = project_reference(vertex);
                    min = min.min(pixel);
                    max = max.max(pixel);
                }
            }
        }
        min -= Vec2::splat(3.0);
        max += Vec2::splat(3.0);
        let size = max - min;
        let mid = (min + max) * 0.5;
        let ray = reference_ray(mid);
        let shadow_center = (shadow_center - min) / size;
        let shadow_radius = shadow_radius / size;
        (
            Rect::from_corners(min * REFERENCE_SCALE, max * REFERENCE_SCALE),
            MahjongDieMaterial {
                ray_origin: (inverse * (CAMERA_POSITION - center) / DIE_HALF_EDGE).extend(0.0),
                ray_center: (inverse * ray).extend(0.0),
                ray_right: (inverse * (reference_ray(mid + Vec2::X * size.x) - ray)).extend(0.0),
                ray_down: (inverse * (reference_ray(mid + Vec2::Y * size.y) - ray)).extend(0.0),
                light: (inverse * Vec3::new(-0.32, 1.0, 0.28).normalize()).extend(0.0),
                shadow: Vec4::new(
                    shadow_center.x,
                    shadow_center.y,
                    shadow_radius.x,
                    shadow_radius.y,
                ),
                red: u32::from(self.red),
            },
        )
    }
}

fn sync_mahjong_dice(
    mut commands: Commands,
    client: Option<Res<ClientResource>>,
    transition: Res<StartGameSeatTransition>,
    tables: Query<Entity, With<MahjongTableRoot>>,
    mut rolls: Query<(Entity, &ChildOf, &mut MahjongDiceRoll)>,
    mut materials: ResMut<Assets<MahjongDieMaterial>>,
) {
    let phase = client.as_deref().map(|client| client.0.model().phase());
    let mut desired = match (phase, tables.single()) {
        (Some(ClientPhaseRef::Playing(GameSnapshot::Mahjong(game))), Ok(table))
            if !transition.is_active_for(game.match_id) =>
        {
            match game.phase {
                MahjongPhaseView::RollingDice => Some((table, game, true)),
                MahjongPhaseView::Dealing { batch } if batch <= LAST_ROLL_BATCH => {
                    Some((table, game, false))
                }
                _ => None,
            }
        }
        _ => None,
    };
    for (entity, parent, mut roll) in &mut rolls {
        if let Some((table, game, rolling)) = desired
            && parent.parent() == table
            && roll.hand == (game.match_id, game.sequence_index)
        {
            if !rolling && roll.elapsed < ROLL_DURATION {
                roll.elapsed = ROLL_DURATION;
            }
            desired = None;
        } else {
            commands.entity(entity).despawn();
        }
    }
    if let Some((table, game, rolling)) = desired {
        spawn_mahjong_dice(&mut commands, table, game, rolling, &mut materials);
    }
}

fn spawn_mahjong_dice(
    commands: &mut Commands,
    table: Entity,
    game: &MahjongSnapshot,
    rolling: bool,
    materials: &mut Assets<MahjongDieMaterial>,
) {
    // The table provides a coordinate frame. The dice plugin owns this subtree.
    let layer = commands
        .spawn((
            ChildOf(table),
            MahjongProjectionFrame::canvas(false),
            GlobalZIndex(98),
            MahjongDiceRoll {
                hand: (game.match_id, game.sequence_index),
                elapsed: if rolling { 0.0 } else { ROLL_DURATION },
            },
        ))
        .id();
    let own_seat = game
        .players
        .iter()
        .find(|player| player.player.id == game.you)
        .map_or(0, |player| player.seat.0);
    let dealer_relative = (game.dealer.0 + 4 - own_seat) % 4;
    let source = match dealer_relative {
        0 => Vec2::new(0.0, -12.0),
        1 => Vec2::new(13.0, 0.0),
        2 => Vec2::new(0.0, 12.0),
        _ => Vec2::new(-13.0, 0.0),
    };
    let center = ground_from_layout(
        MahjongLayout::table_center(),
        Vec2::new(DESIGN_WIDTH, REFERENCE_DESIGN_HEIGHT),
    );
    for (index, face) in game.wall_break.dice().into_iter().enumerate() {
        // These rotations put the face selected by the host on the horizontal top.
        let rest = match face {
            1 => Quat::IDENTITY,
            2 => Quat::from_rotation_x(FRAC_PI_2),
            3 => Quat::from_rotation_z(FRAC_PI_2),
            4 => Quat::from_rotation_z(-FRAC_PI_2),
            5 => Quat::from_rotation_x(-FRAC_PI_2),
            6 => Quat::from_rotation_x(PI),
            _ => unreachable!("the protocol validates die faces"),
        };
        let lane = 2.0 * index as f32 - 1.0;
        let die = MahjongRollingDie {
            source: source + Vec2::X * (lane * 0.48),
            target: center + Vec2::X * (lane * 0.56),
            rest,
            lane,
            red: index == 1,
        };
        let (bounds, material) = die.projection(if rolling { 0.0 } else { 1.0 });
        commands.spawn((
            ChildOf(layer),
            Node {
                position_type: PositionType::Absolute,
                left: px(bounds.min.x),
                top: px(bounds.min.y),
                width: px(bounds.width()),
                height: px(bounds.height()),
                ..default()
            },
            Pickable::IGNORE,
            MaterialNode(materials.add(material)),
            die,
        ));
    }
}

fn animate_mahjong_dice(
    time: Res<Time>,
    assets: Res<MahjongAssets>,
    mut commands: Commands,
    mut sounded_hand: Local<Option<(MatchId, u8)>>,
    mut rolls: Query<(&Children, &mut MahjongDiceRoll, &mut Visibility)>,
    mut dice: Query<(
        &MahjongRollingDie,
        &MaterialNode<MahjongDieMaterial>,
        &mut Node,
    )>,
    mut materials: ResMut<Assets<MahjongDieMaterial>>,
) {
    let end = ROLL_DURATION + HOLD_DURATION;
    for (children, mut roll, mut visibility) in &mut rolls {
        let previous = roll.elapsed;
        if previous >= end {
            continue;
        }
        let update_pose = previous < ROLL_DURATION || roll.is_changed();
        roll.elapsed = (previous + time.delta_secs()).min(end);
        if previous < FIRST_LANDING
            && roll.elapsed >= FIRST_LANDING
            && *sounded_hand != Some(roll.hand)
        {
            *sounded_hand = Some(roll.hand);
            commands.spawn((
                AudioPlayer::new(assets.dice_throw.clone()),
                PlaybackSettings {
                    volume: Volume::Linear(0.48),
                    ..PlaybackSettings::DESPAWN
                },
            ));
        }
        if roll.elapsed >= end {
            *visibility = Visibility::Hidden;
            continue;
        }
        if !update_pose {
            continue;
        }
        let progress = (roll.elapsed / ROLL_DURATION).min(1.0);
        for child in children.iter() {
            let Ok((die, material, mut node)) = dice.get_mut(child) else {
                continue;
            };
            let (bounds, projected) = die.projection(progress);
            node.left = px(bounds.min.x);
            node.top = px(bounds.min.y);
            node.width = px(bounds.width());
            node.height = px(bounds.height());
            if let Some(mut material) = materials.get_mut(&material.0) {
                *material = projected;
            }
        }
    }
}
