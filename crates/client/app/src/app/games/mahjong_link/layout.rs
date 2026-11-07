use crate::app::presentation::mahjong::{MahjongTileGeometry, project_layout, world_layout};
use bevy::prelude::*;
use leocard_mahjong_link::{BOARD_COLUMNS, BOARD_ROWS, LinkCell};

pub(super) const TILE_SCALE: f32 = 1.2;
pub(super) const STEP_X: f32 = 1.28;
const STEP_Y: f32 = 1.65;
const BOARD_FORWARD: f32 = 2.8;

pub(super) fn cell_world(cell: LinkCell) -> Vec2 {
    grid_world(f32::from(cell.row), f32::from(cell.column))
}

fn grid_world(row: f32, column: f32) -> Vec2 {
    Vec2::new(
        (column - (BOARD_COLUMNS - 1) as f32 * 0.5) * STEP_X,
        -(row - (BOARD_ROWS - 1) as f32 * 0.5) * STEP_Y - BOARD_FORWARD,
    )
}

pub(super) fn cell_layout(cell: LinkCell) -> Vec2 {
    world_layout(cell_world(cell))
}

pub(super) fn layout_offset(offset: Vec2) -> Vec2 {
    world_layout(offset) - world_layout(Vec2::ZERO)
}

pub(super) fn project_point(row: f32, column: f32, viewport: Vec2) -> Vec2 {
    let ground = grid_world(row, column);
    project_layout(Vec3::new(ground.x, 0.76, ground.y), viewport)
}

pub(super) fn column_width(row: u8, viewport: Vec2) -> f32 {
    let row = f32::from(row);
    project_point(row, 1.0, viewport).x - project_point(row, 0.0, viewport).x
}

pub(super) fn tile_bounds(cell: LinkCell, offset: Vec2, elevation: f32, viewport: Vec2) -> Rect {
    let ground = cell_world(cell) + offset;
    let half = MahjongTileGeometry::flat_half_size(TILE_SCALE);
    let center = Vec3::new(ground.x, half.y + elevation / 30.0 * TILE_SCALE, ground.y);
    let mut bounds = Rect {
        min: Vec2::splat(f32::INFINITY),
        max: Vec2::splat(f32::NEG_INFINITY),
    };
    for x in [-1.0, 1.0] {
        for y in [-1.0, 1.0] {
            for z in [-1.0, 1.0] {
                let pixel = project_layout(center + half * Vec3::new(x, y, z), viewport);
                bounds.min = bounds.min.min(pixel);
                bounds.max = bounds.max.max(pixel);
            }
        }
    }
    bounds
}

pub(super) fn hitbox_node(bounds: Rect) -> Node {
    Node {
        position_type: PositionType::Absolute,
        left: px(bounds.min.x),
        top: px(bounds.min.y),
        width: px(bounds.width()),
        height: px(bounds.height()),
        ..default()
    }
}
