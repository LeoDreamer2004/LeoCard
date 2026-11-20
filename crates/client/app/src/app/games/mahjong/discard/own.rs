//! 本地弃牌飞行与手牌补位共用一段动画进度。
use super::super::layout::MahjongLayout as Layout;
use super::super::{
    MahjongAssets, MahjongHandTile, MahjongTableRoot, MahjongTileHighlight, MahjongTileMaterial,
    MahjongTileSize, MahjongTileVisual, MahjongUiState, add_mahjong_tile_material,
    mahjong_own_meld_width, mahjong_own_row_left,
};
use crate::app::presentation::ease_out_cubic;
use crate::app::presentation::mahjong::{
    MahjongProjectionViewport, REFERENCE_DESIGN_HEIGHT, camera_canvas, ground_from_layout,
    overlay_tile_scale, world_layout,
};
use bevy::picking::Pickable;
use bevy::prelude::*;
use leocard_mahjong::MahjongTile;
use leocard_ui::UiComposer;

pub(in super::super) const DISCARD_DURATION: f32 = 0.48;
fn hand_tile_advance() -> f32 {
    Layout::own_step()
}

#[derive(Clone, Debug)]
pub(crate) struct MahjongOwnDiscardAnimation {
    pub tile: MahjongTile,
    pub old_hand: Vec<MahjongTile>,
    pub old_drawn: Option<MahjongTile>,
    pub old_meld_count: usize,
    pub discard_index: usize,
    pub river_slot: usize,
    pub elapsed: f32,
    pub(super) table_height: Option<f32>,
    pub(super) table_viewport: Vec2,
}

impl MahjongOwnDiscardAnimation {
    fn movement(&self) -> f32 {
        ease_out_cubic((self.elapsed / DISCARD_DURATION).clamp(0.0, 1.0))
    }

    pub fn hand_shift_transform(&self, start_x: f32) -> UiTransform {
        UiTransform::from_translation(Val2::px(start_x * (1.0 - self.movement()), 0.0))
    }

    fn flight_transform(&self, source_x: f32, target: Vec2) -> UiTransform {
        let source_center = Vec2::new(
            source_x + Layout::own_width() / 2.0,
            REFERENCE_DESIGN_HEIGHT
                - Layout::own_bottom()
                - Layout::OWN_HAND_LIFT
                - Layout::own_height() / 2.0,
        );
        let target_center =
            target + Vec2::new(Layout::RIVER_WIDTH / 2.0, Layout::RIVER_HEIGHT / 2.0);
        let (canvas_scale, mut canvas_origin) = camera_canvas(self.table_viewport);
        canvas_origin.y += self.table_viewport.y - REFERENCE_DESIGN_HEIGHT * canvas_scale;
        let source_center = canvas_origin + source_center * canvas_scale;
        let source_layout = world_layout(ground_from_layout(source_center, self.table_viewport));
        let scale = overlay_tile_scale(
            source_center,
            self.table_viewport,
            Layout::own_width() * canvas_scale,
        );
        let offset = (source_layout - target_center) * (1.0 - self.movement());
        UiTransform {
            translation: Val2::px(offset.x, offset.y),
            rotation: Rot2::radians(Layout::river_tile_rotation(self.tile, 0) * self.movement()),
            scale: Vec2::splat(1.0 + (scale - 1.0) * (1.0 - self.movement())),
        }
    }

    pub fn old_tile_x(&self, tile: MahjongTile) -> Option<f32> {
        let index = self.old_hand.iter().position(|old| *old == tile)?;
        let drawn_index = self
            .old_drawn
            .and_then(|drawn| self.old_hand.iter().position(|old| *old == drawn));
        let regular_count = self.old_hand.len() - usize::from(drawn_index.is_some());
        let left = mahjong_own_row_left(self.old_meld_count, regular_count)
            + self.old_meld_count as f32 * mahjong_own_meld_width();
        if drawn_index == Some(index) {
            Some(left + regular_count as f32 * hand_tile_advance() + Layout::draw_gap())
        } else {
            let regular_index = index - usize::from(drawn_index.is_some_and(|drawn| drawn < index));
            Some(left + regular_index as f32 * hand_tile_advance())
        }
    }

    fn river_position(&self) -> Vec2 {
        Layout::river_tile_center(0, self.river_slot)
            - Vec2::new(Layout::RIVER_WIDTH, Layout::RIVER_HEIGHT) * 0.5
    }
}

#[derive(Component)]
pub(in super::super) struct MahjongDiscardHandShift {
    pub discard_index: usize,
    pub start_x: f32,
    pub drawn: bool,
}

#[derive(Component)]
pub(in super::super) struct MahjongDiscardFlight {
    discard_index: usize,
    table: Entity,
    source_x: f32,
    target: Vec2,
}

#[derive(Component)]
pub(in super::super) struct MahjongDiscardRiverTile(pub usize);

pub(in super::super) fn render_own_discard_flight(
    commands: &mut UiComposer,
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
            elevation: 0.0,
            kind: Some(animation.tile.kind()),
            size: MahjongTileSize::River,
            index: 0,
            highlight: MahjongTileHighlight::None,
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
        .map(|_| animation.flight_transform(source_x, target));
    commands.entity(flight).insert((
        MahjongDiscardFlight {
            discard_index: animation.discard_index,
            table,
            source_x,
            target,
        },
        ZIndex(5),
        if transform.is_some() {
            Visibility::Visible
        } else {
            Visibility::Hidden
        },
        Pickable::IGNORE,
    ));
    if let Some(transform) = transform {
        commands.entity(flight).insert(transform);
    }
}

#[expect(
    clippy::type_complexity,
    clippy::too_many_arguments,
    reason = "the disjoint visual groups share discard progress and the table camera"
)]
pub(in super::super) fn animate_own_discard(
    mut commands: UiComposer,
    time: Res<Time>,
    viewport: Res<MahjongProjectionViewport>,
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
        (Entity, &MahjongDiscardRiverTile, &mut Visibility),
        (With<MahjongDiscardRiverTile>, Without<MahjongDiscardFlight>),
    >,
) {
    for table in &tables {
        if table.size().y > 0.0 {
            ui.last_table_height = Some(table.size().y * table.inverse_scale_factor());
        }
    }
    let Some(animation) = ui.discard_animation.as_mut() else {
        return;
    };
    animation.elapsed += time.delta_secs();
    animation.table_viewport = viewport.size;
    let raw = (animation.elapsed / DISCARD_DURATION).clamp(0.0, 1.0);
    let movement = animation.movement();
    for (entity, shift, tile, mut transform, mut z_index) in &mut shifts {
        if shift.discard_index != animation.discard_index {
            transform.translation = Val2::ZERO;
            *z_index = ZIndex(tile.index);
            commands.entity(entity).remove::<MahjongDiscardHandShift>();
            continue;
        }
        transform.translation = Val2::px(shift.start_x * (1.0 - movement), 0.0);
        *z_index = ZIndex(if shift.drawn { -1 } else { 100 + tile.index });
        if raw >= 1.0 {
            *z_index = ZIndex(tile.index);
            commands.entity(entity).remove::<MahjongDiscardHandShift>();
        }
    }
    for (entity, flight, mut transform, mut visibility) in &mut flights {
        if flight.discard_index != animation.discard_index {
            commands.entity(entity).despawn();
            continue;
        }
        let Ok(table) = tables.get(flight.table) else {
            continue;
        };
        if table.size().y <= 0.0 {
            continue;
        }
        animation.table_height = Some(table.size().y * table.inverse_scale_factor());
        *visibility = Visibility::Visible;
        *transform = animation.flight_transform(flight.source_x, flight.target);
        if raw >= 1.0 {
            commands.entity(entity).despawn();
        }
    }
    for (entity, marker, mut visibility) in &mut river_tiles {
        let finished = marker.0 != animation.discard_index || raw >= 1.0;
        *visibility = if finished {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
        if finished {
            commands.entity(entity).remove::<MahjongDiscardRiverTile>();
        }
    }
    if raw >= 1.0 {
        ui.discard_animation = None;
    }
}
