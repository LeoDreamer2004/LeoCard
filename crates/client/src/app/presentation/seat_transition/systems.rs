//! Move the real in-game player panels out of their former lobby rectangles.

use super::*;
use crate::app::smootherstep;

#[derive(Clone, Copy, Debug)]
pub struct StartGameSeatTransitionVisual {
    pub movement: f32,
}

pub fn start_game_seat_transition_visual(elapsed: f32) -> StartGameSeatTransitionVisual {
    StartGameSeatTransitionVisual {
        movement: smootherstep((elapsed / START_GAME_SEAT_MOVE_DURATION).clamp(0.0, 1.0)),
    }
}

pub fn attach_start_game_seat_transition(
    commands: &mut Commands,
    panel: Entity,
    player: PlayerId,
    active: bool,
) {
    commands.entity(panel).insert((
        GameSeatTransitionTarget(player),
        GameSeatTransitionPose::default(),
        UiTransform::IDENTITY,
    ));
    if active {
        commands.entity(panel).insert(Visibility::Hidden);
    }
}

fn initial_transition_pose(
    source: &LobbySeatTransitionSnapshot,
    target_node: &ComputedNode,
    target_global: &UiGlobalTransform,
) -> GameSeatTransitionPose {
    let target_center = target_global.to_scale_angle_translation().2;
    let target_size = target_node.size() * target_node.inverse_scale_factor();
    let start_translation =
        (source.center_global - target_center) * target_node.inverse_scale_factor();
    let start_scale = Vec2::new(source.size.x / target_size.x, source.size.y / target_size.y)
        .clamp(Vec2::splat(0.1), Vec2::splat(10.0));

    GameSeatTransitionPose {
        start_translation,
        start_scale,
        initialized: true,
    }
}

fn interpolated_transform(pose: GameSeatTransitionPose, movement: f32) -> UiTransform {
    UiTransform {
        translation: Val2::px(
            pose.start_translation.x * (1.0 - movement),
            pose.start_translation.y * (1.0 - movement),
        ),
        scale: pose.start_scale.lerp(Vec2::ONE, movement),
        ..default()
    }
}

pub fn animate_start_game_seat_transition(
    time: Res<Time>,
    mut transition: ResMut<StartGameSeatTransition>,
    mut targets: Query<(
        &GameSeatTransitionTarget,
        &ComputedNode,
        &UiGlobalTransform,
        &mut Visibility,
        &mut UiTransform,
        &mut GameSeatTransitionPose,
    )>,
) {
    if transition.match_id.is_none() || transition.seats.is_empty() {
        return;
    }

    // The new UI needs one layout pass before its final rectangles are usable. Keep
    // its panels hidden until every panel participating in the transition is ready.
    let mut ready = 0;
    for (target, node, global, _, _, mut pose) in &mut targets {
        let Some(source) = transition
            .seats
            .iter()
            .find(|source| source.player == target.0)
        else {
            continue;
        };
        if node.size().min_element() <= 1.0 {
            continue;
        }
        if !pose.initialized {
            *pose = initial_transition_pose(source, node, global);
        }
        ready += 1;
    }
    if ready != transition.seats.len() {
        return;
    }

    let visual = start_game_seat_transition_visual(transition.elapsed);
    for (target, _, _, mut visibility, mut transform, pose) in &mut targets {
        if !transition
            .seats
            .iter()
            .any(|source| source.player == target.0)
        {
            continue;
        }
        *transform = interpolated_transform(*pose, visual.movement);
        *visibility = Visibility::Visible;
    }

    // Applying the zero-time pose before advancing guarantees that the first visible
    // frame exactly matches the old lobby rectangle instead of jumping ahead.
    if transition.elapsed >= START_GAME_SEAT_MOVE_DURATION {
        for (target, _, _, mut visibility, mut transform, _) in &mut targets {
            if transition
                .seats
                .iter()
                .any(|source| source.player == target.0)
            {
                *transform = UiTransform::IDENTITY;
                *visibility = Visibility::Visible;
            }
        }
        transition.clear();
    } else {
        transition.elapsed =
            (transition.elapsed + time.delta_secs()).min(START_GAME_SEAT_MOVE_DURATION);
    }
}
