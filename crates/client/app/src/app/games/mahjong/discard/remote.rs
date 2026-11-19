use super::super::MahjongUiState;
use super::super::layout::MahjongLayout as Layout;
use super::DISCARD_DURATION;
use crate::app::presentation::ease_out_cubic;
use bevy::picking::Pickable;
use bevy::prelude::*;
use leocard_game_common::PlayerId;
use leocard_mahjong::MahjongTile;
use leocard_ui::UiComposer;

#[derive(Clone, Debug)]
pub(crate) struct MahjongRemoteDiscardAnimation {
    pub player: PlayerId,
    pub tile: MahjongTile,
    pub discard_index: usize,
    pub from_drawn: bool,
    pub elapsed: f32,
    pub(super) pose: Option<MahjongRemoteFlightPose>,
}

#[derive(Component)]
pub(in super::super) struct MahjongRemoteHandShift {
    pub discard_index: usize,
    pub start_x: f32,
    pub end_x: f32,
}

#[derive(Component)]
pub(in super::super) struct MahjongRemoteDiscardSource {
    discard_index: usize,
    tile: MahjongTile,
}

#[derive(Component)]
pub(in super::super) struct MahjongRemoteDiscardRiverTile {
    discard_index: usize,
    tile: MahjongTile,
    rest_rotation: f32,
}

#[derive(Clone, Copy, Debug)]
pub(super) struct MahjongRemoteFlightPose {
    offset: Vec2,
    rotation: f32,
    scale: Vec2,
}

impl MahjongRemoteFlightPose {
    fn transform(self, movement: f32, rest_rotation: f32) -> UiTransform {
        let offset = self.offset * (1.0 - movement);
        UiTransform {
            translation: Val2::px(offset.x, offset.y),
            rotation: Rot2::radians(self.rotation + (rest_rotation - self.rotation) * movement),
            scale: self.scale.lerp(Vec2::ONE, movement),
        }
    }
}

impl MahjongRemoteDiscardAnimation {
    fn movement(&self) -> f32 {
        ease_out_cubic((self.elapsed / DISCARD_DURATION).clamp(0.0, 1.0))
    }

    pub fn hand_shift_transform(&self, start_x: f32, end_x: f32) -> UiTransform {
        UiTransform::from_translation(Val2::px(start_x + (end_x - start_x) * self.movement(), 0.0))
    }

    pub fn render_source(&self, commands: &mut UiComposer, hand: Entity, left: f32) {
        commands.keyed_child(
            hand,
            format!("discard-source:{}:{:?}", self.discard_index, self.tile),
            (
                Node {
                    position_type: PositionType::Absolute,
                    left: px(left),
                    bottom: px(0),
                    width: px(Layout::RIVER_WIDTH),
                    height: px(Layout::RIVER_HEIGHT),
                    ..default()
                },
                MahjongRemoteDiscardSource {
                    discard_index: self.discard_index,
                    tile: self.tile,
                },
                Pickable::IGNORE,
            ),
        );
    }
}

pub(in super::super) fn render_remote_discard_river_tile(
    commands: &mut UiComposer,
    entity: Entity,
    discard_index: usize,
    tile: MahjongTile,
    rest_rotation: f32,
    animation: Option<&MahjongRemoteDiscardAnimation>,
) {
    let animation = animation.filter(|a| a.discard_index == discard_index && a.tile == tile);
    let transform = animation
        .and_then(|a| {
            a.pose
                .map(|pose| pose.transform(a.movement(), rest_rotation))
        })
        .unwrap_or_else(|| UiTransform::from_rotation(Rot2::radians(rest_rotation)));
    commands.entity(entity).insert((
        MahjongRemoteDiscardRiverTile {
            discard_index,
            tile,
            rest_rotation,
        },
        transform,
        if animation.is_some_and(|a| a.pose.is_none()) {
            Visibility::Hidden
        } else {
            Visibility::Visible
        },
    ));
}

// Measure fresh layout only once. The flight then belongs to the river, so later
// hand shifts and page recomposition cannot carry its landing position away.
pub(in super::super) fn capture_remote_discard_source(
    mut ui: ResMut<MahjongUiState>,
    sources: Query<(
        &MahjongRemoteDiscardSource,
        &ComputedNode,
        &UiGlobalTransform,
    )>,
    targets: Query<(
        &MahjongRemoteDiscardRiverTile,
        &ComputedNode,
        &UiGlobalTransform,
        &ChildOf,
    )>,
    parents: Query<&UiGlobalTransform>,
) {
    let Some(animation) = ui
        .remote_discard_animation
        .as_mut()
        .filter(|a| a.pose.is_none())
    else {
        return;
    };
    let Some((_, source_node, source)) = sources.iter().find(|(marker, _, _)| {
        marker.discard_index == animation.discard_index && marker.tile == animation.tile
    }) else {
        return;
    };
    let Some((_, target_node, target, parent)) = targets.iter().find(|(marker, _, _, _)| {
        marker.discard_index == animation.discard_index && marker.tile == animation.tile
    }) else {
        return;
    };
    if source_node.size().min_element() <= 1.0 || target_node.size().min_element() <= 1.0 {
        return;
    }
    let Ok(parent) = parents.get(parent.parent()) else {
        return;
    };
    let Some(inverse) = parent.try_inverse() else {
        return;
    };
    let source_axis = inverse.transform_vector2(source.matrix2.x_axis);
    let source_y_axis = inverse.transform_vector2(source.matrix2.y_axis);
    animation.pose = Some(MahjongRemoteFlightPose {
        offset: inverse.transform_vector2(
            source.to_scale_angle_translation().2 - target.to_scale_angle_translation().2,
        ) * target_node.inverse_scale_factor(),
        rotation: source_axis.y.atan2(source_axis.x),
        scale: source_node.size() / target_node.size()
            * Vec2::new(source_axis.length(), source_y_axis.length()),
    });
}

pub(in super::super) fn animate_remote_discard(
    mut commands: UiComposer,
    time: Res<Time>,
    mut ui: ResMut<MahjongUiState>,
    mut river_tiles: Query<(
        Entity,
        &MahjongRemoteDiscardRiverTile,
        &mut UiTransform,
        &mut Visibility,
    )>,
    mut hand_shifts: Query<
        (Entity, &MahjongRemoteHandShift, &mut UiTransform),
        Without<MahjongRemoteDiscardRiverTile>,
    >,
) {
    if let Some(animation) = ui.remote_discard_animation.as_mut()
        && animation.pose.is_some()
    {
        animation.elapsed += time.delta_secs();
    }
    let animation = ui.remote_discard_animation.as_ref();
    let finished = animation.is_none_or(|a| a.elapsed >= DISCARD_DURATION);
    for (_, marker, mut transform, mut visibility) in &mut river_tiles {
        let active =
            animation.filter(|a| a.discard_index == marker.discard_index && a.tile == marker.tile);
        if let Some(animation) = active {
            if let Some(pose) = animation.pose {
                *transform = pose.transform(animation.movement(), marker.rest_rotation);
                *visibility = Visibility::Visible;
            } else {
                *transform = UiTransform::from_rotation(Rot2::radians(marker.rest_rotation));
                *visibility = Visibility::Hidden;
            }
        }
        if active.is_none() || finished {
            *transform = UiTransform::from_rotation(Rot2::radians(marker.rest_rotation));
            *visibility = Visibility::Visible;
        }
    }
    for (entity, shift, mut transform) in &mut hand_shifts {
        let active = animation.filter(|a| a.discard_index == shift.discard_index);
        transform.translation = if let Some(animation) = active.filter(|_| !finished) {
            animation
                .hand_shift_transform(shift.start_x, shift.end_x)
                .translation
        } else {
            Val2::ZERO
        };
        if active.is_none() || finished {
            commands.entity(entity).remove::<MahjongRemoteHandShift>();
        }
    }
    if finished {
        ui.remote_discard_animation = None;
    }
}
