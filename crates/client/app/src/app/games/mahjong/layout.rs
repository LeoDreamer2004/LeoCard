//! 共用标定后的桌面坐标，Android 扩大前景手牌和操作区。
use crate::app::presentation::DESIGN_WIDTH;
use crate::app::presentation::mahjong::{
    MahjongTileGeometry, REFERENCE_SIZE, project_reference, world_layout,
};
use bevy::prelude::*;
use leocard_mahjong::MahjongTile;
use std::f32::consts;

pub(super) struct MahjongLayout;

impl MahjongLayout {
    pub const OWN_RIM_RATIO: f32 = 0.115;
    pub const OWN_MELD_GAP: f32 = 8.0;
    pub const REMOTE_MELD_GAP: f32 = 12.0;
    pub const REMOTE_HAND_GAP: f32 = 18.0;
    pub const WALL_SCALE: f32 = 0.62;
    pub const WALL_DISTANCE: f32 = 9.8;
    pub const OPPOSITE_WALL_DISTANCE: f32 = 8.8;
    pub const CENTER_FORWARD: f32 = 0.6;
    pub const OWN_HAND_LIFT: f32 = 12.0;
    const REFERENCE_SCALE: f32 = DESIGN_WIDTH / REFERENCE_SIZE.x;
    pub const STATUS_SIZE: Vec2 =
        Vec2::new(390.0 * Self::REFERENCE_SCALE, 254.5 * Self::REFERENCE_SCALE);
    pub const STATUS_TOP_RATIO: f32 = 352.0 / 390.0;

    pub const BUTTON_WIDTH: f32 = if cfg!(target_os = "android") {
        120.0
    } else {
        96.0
    };
    pub const BUTTON_HEIGHT: f32 = if cfg!(target_os = "android") {
        64.0
    } else {
        54.0
    };
    pub const BUTTON_FONT: f32 = if cfg!(target_os = "android") {
        42.0
    } else {
        34.0
    };

    pub const OWN_CENTER: f32 = if cfg!(target_os = "android") {
        574.12
    } else {
        556.3
    };
    pub const OWN_BOTTOM: f32 = if cfg!(target_os = "android") {
        8.4
    } else {
        7.5
    };
    pub const OWN_WIDTH: f32 = if cfg!(target_os = "android") {
        76.56
    } else {
        60.9
    };
    pub const OWN_HEIGHT: f32 =
        Self::OWN_WIDTH * (1.0 / MahjongTileGeometry::FACE_ASPECT_RATIO + Self::OWN_RIM_RATIO);
    pub const OWN_STEP: f32 = if cfg!(target_os = "android") {
        78.0
    } else {
        62.0
    };
    pub const OWN_ROW_HEIGHT: f32 = Self::OWN_HEIGHT + 18.0;
    pub const OWN_ROW_WIDTH: f32 = if cfg!(target_os = "android") {
        1150.0
    } else {
        1000.0
    };
    pub const DRAW_GAP: f32 = if cfg!(target_os = "android") {
        24.0
    } else {
        18.7
    };

    pub const MINI_WIDTH: f32 = 16.0;
    pub const MINI_HEIGHT: f32 = Self::MINI_WIDTH / MahjongTileGeometry::FACE_ASPECT_RATIO;
    pub const MINI_STEP: f32 = 16.5;
    pub const REMOTE_HEIGHT: f32 = 30.0;
    pub const RIVER_WIDTH: f32 = 16.0;
    pub const RIVER_HEIGHT: f32 = Self::RIVER_WIDTH / MahjongTileGeometry::FACE_ASPECT_RATIO;
    pub const RIVER_STEP: f32 = 16.5;
    pub const RIVER_COLUMNS: usize = 6;
    pub const RIVER_ROWS_HEIGHT: f32 = Self::RIVER_HEIGHT * 3.0 + 1.5;
    pub const RIVER_ROWS_WIDTH: f32 = 99.0;

    pub fn status_origin() -> Vec2 {
        let offset = project_reference(Vec3::new(0.0, 0.0, -Self::CENTER_FORWARD))
            - project_reference(Vec3::ZERO);
        (Vec2::new(900.0, 349.5) + offset) * Self::REFERENCE_SCALE
    }

    pub fn status_point(uv: Vec2) -> Vec2 {
        let top_ratio = Self::STATUS_TOP_RATIO;
        let denominator = 1.0 - (1.0 - top_ratio) * uv.y;
        Vec2::new(
            0.5 + (uv.x - 0.5) * top_ratio / denominator,
            uv.y * top_ratio / denominator,
        ) * Self::STATUS_SIZE
    }

    pub fn table_center() -> Vec2 {
        Self::status_origin() + Self::status_point(Vec2::splat(0.5))
    }

    pub fn river_tile_rotation(tile: MahjongTile, relative: u8) -> f32 {
        // Physical tile identity keeps the small variation stable across snapshots and claimed discards.
        let identity = tile.kind().index34().unwrap_or(34) as u32 * 4 + u32::from(tile.copy());
        let hash = (identity + u32::from(relative) * 149 + 1).wrapping_mul(2_654_435_761);
        ((hash >> 16) % 2001) as f32 / 1000.0 * 0.014 - 0.014
    }

    pub const fn river_tile_layer(relative: u8, index: usize) -> i32 {
        let row = (index / Self::RIVER_COLUMNS) as i32;
        let column = (index % Self::RIVER_COLUMNS) as i32;
        // Sort by screen depth after rotating the river, then by its horizontal position.
        match relative {
            0 => row * 32 + column,
            1 => -column * 32 + row,
            2 => -row * 32 - column,
            _ => column * 32 - row,
        }
    }

    pub fn river_origin(relative: u8) -> Vec2 {
        let mut first = match relative {
            0 => Vec2::new(-2.7078, -4.1388),
            1 => Vec2::new(4.0517, -2.7940),
            2 => Vec2::new(2.6217, 3.9348),
            _ => Vec2::new(-4.1415, 2.8361),
        };
        first.y -= Self::CENTER_FORWARD;
        let half = Vec2::new(Self::RIVER_ROWS_WIDTH, Self::RIVER_ROWS_HEIGHT) * 0.5;
        let tile_center = Vec2::new(Self::RIVER_WIDTH, Self::RIVER_HEIGHT) * 0.5;
        world_layout(first)
            - half
            - Rot2::radians(-f32::from(relative) * consts::FRAC_PI_2) * (tile_center - half)
    }

    pub fn river_tile_center(relative: u8, index: usize) -> Vec2 {
        let half = Vec2::new(Self::RIVER_ROWS_WIDTH, Self::RIVER_ROWS_HEIGHT) * 0.5;
        let local = Vec2::new(Self::RIVER_WIDTH, Self::RIVER_HEIGHT) * 0.5
            + Vec2::new(
                (index % Self::RIVER_COLUMNS) as f32 * Self::RIVER_STEP,
                (index / Self::RIVER_COLUMNS) as f32
                    * (Self::RIVER_HEIGHT + Self::RIVER_STEP - Self::RIVER_WIDTH),
            );
        Self::river_origin(relative)
            + half
            + Rot2::radians(-f32::from(relative) * consts::FRAC_PI_2) * (local - half)
    }
}
