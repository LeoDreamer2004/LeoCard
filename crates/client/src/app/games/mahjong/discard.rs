//! 自己弃牌后，牌河飞行与手牌补位共用一段动画进度。

use super::{
    MahjongAssets, MahjongHandTile, MahjongTileMaterial, MahjongTileSize, MahjongTileVisual,
    MahjongUiState, add_mahjong_tile_material, mahjong_own_row_left,
};
use crate::app::presentation::ease_out_cubic;
use bevy::prelude::*;
use bevy::ui::FocusPolicy;
use leocard_mahjong::MahjongTile;
use leocard_protocol::{MahjongSnapshot, PlayerId};

const DISCARD_DURATION: f32 = 0.48;
const HAND_TILE_ADVANCE: f32 = 50.0;

#[derive(Component)]
pub(super) struct MahjongTableRoot;

#[derive(Clone, Debug)]
pub(crate) struct MahjongOwnDiscardAnimation {
    pub tile: MahjongTile,
    pub old_hand: Vec<MahjongTile>,
    pub old_drawn: Option<MahjongTile>,
    pub old_meld_count: usize,
    pub discard_index: usize,
    pub river_slot: usize,
    pub elapsed: f32,
    table_height: Option<f32>,
}

#[derive(Clone, Debug)]
pub(crate) struct MahjongRemoteDiscardAnimation {
    pub player: PlayerId,
    pub tile: MahjongTile,
    pub discard_index: usize,
    pub from_drawn: bool,
    pub elapsed: f32,
    pose: Option<MahjongRemoteFlightPose>,
}

impl MahjongOwnDiscardAnimation {
    fn movement(&self) -> f32 {
        ease_out_cubic((self.elapsed / DISCARD_DURATION).clamp(0.0, 1.0))
    }

    pub fn hand_shift_transform(&self, start_x: f32) -> UiTransform {
        UiTransform::from_translation(Val2::px(start_x * (1.0 - self.movement()), 0.0))
    }

    fn flight_transform(&self, source_x: f32, target: Vec2, table_height: f32) -> UiTransform {
        let source_center = Vec2::new(source_x + 28.0, table_height - 8.0 - 38.0);
        let target_center = target + Vec2::new(16.5, 22.5);
        let offset = (source_center - target_center) * (1.0 - self.movement());
        UiTransform {
            translation: Val2::px(offset.x, offset.y),
            scale: Vec2::splat(1.0 + (56.0 / 33.0 - 1.0) * (1.0 - self.movement())),
            ..default()
        }
    }

    pub fn old_tile_x(&self, tile: MahjongTile) -> Option<f32> {
        let index = self.old_hand.iter().position(|old| *old == tile)?;
        let drawn_index = self
            .old_drawn
            .and_then(|drawn| self.old_hand.iter().position(|old| *old == drawn));
        let regular_count = self.old_hand.len() - usize::from(drawn_index.is_some());
        let left = mahjong_own_row_left(self.old_meld_count, regular_count)
            + self.old_meld_count as f32 * super::MAHJONG_OWN_MELD_WIDTH;
        if drawn_index == Some(index) {
            Some(left + regular_count as f32 * HAND_TILE_ADVANCE + 22.0)
        } else {
            let regular_index = index - usize::from(drawn_index.is_some_and(|drawn| drawn < index));
            Some(left + regular_index as f32 * HAND_TILE_ADVANCE)
        }
    }

    fn river_position(&self) -> Vec2 {
        Vec2::new(
            549.0 + (self.river_slot % 6) as f32 * 30.0,
            382.0 + (self.river_slot / 6) as f32 * 42.0,
        )
    }
}

impl MahjongRemoteDiscardAnimation {
    fn movement(&self) -> f32 {
        ease_out_cubic((self.elapsed / DISCARD_DURATION).clamp(0.0, 1.0))
    }

    pub fn hand_shift_transform(&self, start_x: f32, end_x: f32) -> UiTransform {
        UiTransform::from_translation(Val2::px(start_x + (end_x - start_x) * self.movement(), 0.0))
    }

    pub(super) fn ghost_visual(
        &self,
        source_offset_x: f32,
    ) -> (MahjongRemoteDiscardGhost, UiTransform) {
        let transform = self.pose.map_or_else(
            || UiTransform::from_translation(Val2::px(source_offset_x, 0.0)),
            |pose| pose.transform(self.movement()),
        );
        (
            MahjongRemoteDiscardGhost {
                pose: self.pose,
                source_offset_x,
            },
            transform,
        )
    }
}

#[derive(Component)]
pub(super) struct MahjongDiscardHandShift {
    pub start_x: f32,
    pub drawn: bool,
}

#[derive(Component)]
pub(super) struct MahjongDiscardFlight {
    table: Entity,
    source_x: f32,
    target: Vec2,
}

#[derive(Component)]
pub(super) struct MahjongDiscardRiverTile;

#[derive(Component)]
pub(super) struct MahjongRemoteDiscardRiverTile;

#[derive(Component)]
pub(super) struct MahjongRemoteDiscardSpacer;

#[derive(Component)]
pub(super) struct MahjongRemoteHandShift {
    pub start_x: f32,
    pub end_x: f32,
}

#[derive(Component)]
pub(super) struct MahjongRemoteDiscardGhost {
    pose: Option<MahjongRemoteFlightPose>,
    source_offset_x: f32,
}

#[derive(Clone, Copy, Debug)]
struct MahjongRemoteFlightPose {
    source_offset_x: f32,
    delta: Vec2,
    rotation: f32,
    scale: Vec2,
}

impl MahjongRemoteFlightPose {
    fn transform(self, movement: f32) -> UiTransform {
        let offset = self.delta * movement;
        UiTransform {
            translation: Val2::px(self.source_offset_x + offset.x, offset.y),
            rotation: Rot2::radians(self.rotation * movement),
            scale: Vec2::ONE.lerp(self.scale, movement),
        }
    }
}

pub(super) fn observe_discard_animations(ui: &mut MahjongUiState, game: &MahjongSnapshot) {
    let previous = &ui.observed_table.state;
    if let Some(previous_count) = previous.discard_count
        && game.discards.len() == previous_count + 1
        && previous.hand.len() == game.your_hand.len() + 1
        && let Some(discard) = game.discards.last()
        && discard.player == game.you
        && discard.claimed_by.is_none()
        && previous.hand.contains(&discard.tile)
        && !game.your_hand.contains(&discard.tile)
    {
        let river_slot = game
            .discards
            .iter()
            .filter(|entry| entry.player == game.you && entry.claimed_by.is_none())
            .count()
            - 1;
        ui.discard_animation = Some(MahjongOwnDiscardAnimation {
            tile: discard.tile,
            old_hand: previous.hand.clone(),
            old_drawn: previous.drawn_tile,
            old_meld_count: previous.meld_count,
            discard_index: game.discards.len() - 1,
            river_slot,
            elapsed: 0.0,
            table_height: ui.last_table_height,
        });
    } else if ui.discard_animation.as_ref().is_some_and(|active| {
        game.discards
            .get(active.discard_index)
            .is_none_or(|discard| discard.claimed_by.is_some())
    }) {
        ui.discard_animation = None;
    }
    if let Some(previous_count) = previous.discard_count
        && game.discards.len() == previous_count + 1
        && let Some(discard) = game.discards.last()
        && discard.player != game.you
        && discard.claimed_by.is_none()
        && game
            .players
            .iter()
            .any(|player| player.id == discard.player)
    {
        ui.remote_discard_animation = Some(MahjongRemoteDiscardAnimation {
            player: discard.player,
            tile: discard.tile,
            discard_index: game.discards.len() - 1,
            from_drawn: discard.from_drawn,
            elapsed: 0.0,
            pose: None,
        });
    } else if ui.remote_discard_animation.as_ref().is_some_and(|active| {
        game.discards
            .get(active.discard_index)
            .is_none_or(|discard| discard.claimed_by.is_some())
    }) {
        ui.remote_discard_animation = None;
    }
}

pub(super) fn render_own_discard_flight(
    commands: &mut Commands,
    table: Entity,
    animation: Option<&MahjongOwnDiscardAnimation>,
    assets: &MahjongAssets,
    materials: &mut Assets<MahjongTileMaterial>,
) {
    let Some(animation) = animation else { return };
    let Some(source_x) = animation.old_tile_x(animation.tile) else {
        return;
    };
    let target = animation.river_position();
    let flight = add_mahjong_tile_material(
        commands,
        table,
        MahjongTileVisual {
            kind: Some(animation.tile.kind()),
            size: MahjongTileSize::River,
            index: 0,
            highlighted: false,
            deal: None,
            relative: 0,
        },
        assets,
        materials,
    );
    commands
        .entity(flight)
        .entry::<Node>()
        .and_modify(move |mut node| {
            node.position_type = PositionType::Absolute;
            node.left = px(target.x);
            node.top = px(target.y);
            node.margin = UiRect::ZERO;
        });
    let transform = animation
        .table_height
        .map(|height| animation.flight_transform(source_x, target, height));
    commands.entity(flight).insert((
        MahjongDiscardFlight {
            table,
            source_x,
            target,
        },
        ZIndex(900),
        if transform.is_some() {
            Visibility::Visible
        } else {
            Visibility::Hidden
        },
        FocusPolicy::Pass,
    ));
    if let Some(transform) = transform {
        commands.entity(flight).insert(transform);
    }
}

#[expect(
    clippy::type_complexity,
    reason = "the three visual groups share one discard progress"
)]
pub(super) fn animate_own_discard(
    mut commands: Commands,
    time: Res<Time>,
    mut ui: ResMut<MahjongUiState>,
    tables: Query<&ComputedNode, With<MahjongTableRoot>>,
    mut shifts: Query<
        (
            Entity,
            &MahjongDiscardHandShift,
            &MahjongHandTile,
            &mut UiTransform,
            &mut ZIndex,
        ),
        Without<MahjongDiscardFlight>,
    >,
    mut flights: Query<(
        Entity,
        &MahjongDiscardFlight,
        &mut UiTransform,
        &mut Visibility,
    )>,
    mut river_tiles: Query<
        (Entity, &mut Visibility),
        (With<MahjongDiscardRiverTile>, Without<MahjongDiscardFlight>),
    >,
) {
    for table in &tables {
        if table.size().y > 0.0 {
            ui.last_table_height = Some(table.size().y);
        }
    }
    let Some(animation) = ui.discard_animation.as_mut() else {
        return;
    };
    animation.elapsed += time.delta_secs();
    let raw = (animation.elapsed / DISCARD_DURATION).clamp(0.0, 1.0);
    let movement = animation.movement();
    for (entity, shift, tile, mut transform, mut z_index) in &mut shifts {
        transform.translation = Val2::px(shift.start_x * (1.0 - movement), 0.0);
        *z_index = ZIndex(if shift.drawn { -1 } else { 100 + tile.index });
        if raw >= 1.0 {
            *z_index = ZIndex(tile.index);
            commands.entity(entity).remove::<MahjongDiscardHandShift>();
        }
    }
    for (entity, flight, mut transform, mut visibility) in &mut flights {
        let Ok(table) = tables.get(flight.table) else {
            continue;
        };
        if table.size().y <= 0.0 {
            continue;
        }
        animation.table_height = Some(table.size().y);
        *visibility = Visibility::Visible;
        *transform = animation.flight_transform(flight.source_x, flight.target, table.size().y);
        if raw >= 1.0 {
            commands.entity(entity).despawn();
        }
    }
    for (entity, mut visibility) in &mut river_tiles {
        *visibility = if raw >= 1.0 {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
        if raw >= 1.0 {
            commands.entity(entity).remove::<MahjongDiscardRiverTile>();
        }
    }
    if raw >= 1.0 {
        ui.discard_animation = None;
    }
}

fn unrotate(vector: Vec2, angle: f32) -> Vec2 {
    let (sin, cos) = angle.sin_cos();
    Vec2::new(
        vector.x * cos + vector.y * sin,
        -vector.x * sin + vector.y * cos,
    )
}

pub(super) fn animate_remote_discard(
    mut commands: Commands,
    time: Res<Time>,
    mut ui: ResMut<MahjongUiState>,
    mut ghosts: Query<(
        Entity,
        &ComputedNode,
        &UiGlobalTransform,
        &mut UiTransform,
        &mut MahjongRemoteDiscardGhost,
    )>,
    mut river_tiles: Query<
        (Entity, &ComputedNode, &UiGlobalTransform, &mut Visibility),
        With<MahjongRemoteDiscardRiverTile>,
    >,
    spacers: Query<Entity, With<MahjongRemoteDiscardSpacer>>,
    mut hand_shifts: Query<
        (Entity, &MahjongRemoteHandShift, &mut UiTransform),
        Without<MahjongRemoteDiscardGhost>,
    >,
) {
    let Some(animation) = ui.remote_discard_animation.as_mut() else {
        return;
    };
    animation.elapsed += time.delta_secs();
    let raw = (animation.elapsed / DISCARD_DURATION).clamp(0.0, 1.0);
    let movement = animation.movement();
    let Some((target_entity, target_node, target_global, mut target_visibility)) =
        river_tiles.iter_mut().next()
    else {
        return;
    };
    for (entity, source_node, source_global, mut transform, mut ghost) in &mut ghosts {
        if source_node.size().min_element() <= 1.0 || target_node.size().min_element() <= 1.0 {
            continue;
        }
        if ghost.pose.is_none() {
            let (_, source_angle, source_center) = source_global.to_scale_angle_translation();
            let (_, target_angle, target_center) = target_global.to_scale_angle_translation();
            let pose = MahjongRemoteFlightPose {
                source_offset_x: ghost.source_offset_x,
                delta: unrotate(target_center - source_center, source_angle)
                    * source_node.inverse_scale_factor(),
                rotation: target_angle - source_angle,
                scale: target_node.size() / source_node.size(),
            };
            ghost.pose = Some(pose);
            animation.pose = Some(pose);
        }
        let Some(pose) = ghost.pose else { continue };
        *transform = pose.transform(movement);
        if raw >= 1.0 {
            commands.entity(entity).despawn();
        }
    }
    for (entity, shift, mut transform) in &mut hand_shifts {
        transform.translation = Val2::px(
            if raw >= 1.0 {
                0.0
            } else {
                shift.start_x + (shift.end_x - shift.start_x) * movement
            },
            0.0,
        );
        if raw >= 1.0 {
            commands.entity(entity).remove::<MahjongRemoteHandShift>();
        }
    }
    if raw >= 1.0 {
        *target_visibility = Visibility::Visible;
        for entity in &spacers {
            commands.entity(entity).despawn();
        }
        commands
            .entity(target_entity)
            .remove::<MahjongRemoteDiscardRiverTile>();
        ui.remote_discard_animation = None;
    }
}
