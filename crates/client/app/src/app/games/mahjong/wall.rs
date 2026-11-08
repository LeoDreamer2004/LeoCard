use super::layout::MahjongLayout as Layout;
use super::{MahjongAssets, MahjongTileMaterial, attach_mahjong_tile_surface, mahjong_local_light};
use crate::app::presentation::mahjong::world_layout;
use bevy::picking::Pickable;
use bevy::prelude::*;
use leocard_mahjong::{MahjongWallBreak, mahjong_deal_wall_gap};
use leocard_protocol::{MahjongPhaseView, MahjongSnapshot};
use leocard_ui::{UiComposer, spawn_node};
use std::f32::consts;

pub(super) fn render_mahjong_wall(
    commands: &mut UiComposer,
    table: Entity,
    game: &MahjongSnapshot,
    own_seat: u8,
    assets: &MahjongAssets,
    materials: &mut Assets<MahjongTileMaterial>,
) {
    let stacks = MahjongWallBreak::STACKS_PER_SIDE;
    let total = MahjongWallBreak::TILE_COUNT;
    let wall_open = !matches!(game.phase, MahjongPhaseView::RollingDice);
    let first = if wall_open {
        game.wall_break.first_tile_index(game.dealer)
    } else {
        0
    };
    let tail = total.saturating_sub(usize::from(game.wall_tail_drawn));
    let gap = match game.phase {
        MahjongPhaseView::Dealing { batch } => mahjong_deal_wall_gap(batch),
        _ => None,
    };
    let head = tail.saturating_sub(usize::from(game.wall_len) + usize::from(gap.is_some()));
    let is_live = |tile| (head..tail).contains(&tile) && gap != Some(tile);
    let width = stacks as f32 * Layout::MINI_STEP;
    for side in 0..4 {
        // Rotate each physical wall into the viewer's clockwise table coordinates.
        let relative = (side + usize::from(own_seat)) % 4;
        let center = world_layout(match relative {
            0 => Vec2::new(0.0, -Layout::WALL_DISTANCE),
            1 => Vec2::new(-Layout::WALL_DISTANCE, 0.0),
            2 => Vec2::new(0.0, Layout::OPPOSITE_WALL_DISTANCE),
            _ => Vec2::new(Layout::WALL_DISTANCE, 0.0),
        });
        let segment = spawn_node(
            commands,
            table,
            Node {
                position_type: PositionType::Absolute,
                left: px(center.x - width * 0.5),
                top: px(center.y - Layout::MINI_HEIGHT * 0.5),
                width: px(width),
                height: px(Layout::MINI_HEIGHT),
                ..default()
            },
            None,
        );
        commands.entity(segment).insert((
            UiTransform {
                rotation: Rot2::radians((relative as f32 - 2.0) * consts::FRAC_PI_2),
                scale: Vec2::splat(Layout::WALL_SCALE),
                ..default()
            },
            ZIndex(1),
            Pickable::IGNORE,
        ));
        for column in 0..stacks {
            let physical_stack = side * stacks + column;
            let first_layer = (physical_stack * 2 + total - first) % total;
            let layers = u8::from(is_live(first_layer)) + u8::from(is_live(first_layer + 1));
            if layers == 0 {
                continue;
            }
            let stack = commands.keyed_child(
                segment,
                column.to_string(),
                (
                    Node {
                        position_type: PositionType::Absolute,
                        left: px(column as f32 * Layout::MINI_STEP),
                        top: px(0),
                        width: px(Layout::MINI_WIDTH),
                        height: px(Layout::MINI_HEIGHT),
                        ..default()
                    },
                    Pickable::IGNORE,
                ),
            );
            let orientation = ((4 - relative) % 4) as u8;
            add_mahjong_wall_stack(commands, stack, layers, orientation, assets, materials);
        }
    }
}

fn add_mahjong_wall_stack(
    commands: &mut UiComposer,
    stack: Entity,
    layers: u8,
    orientation: u8,
    assets: &MahjongAssets,
    materials: &mut Assets<MahjongTileMaterial>,
) {
    let entity = commands.child(
        stack,
        (
            Node {
                position_type: PositionType::Absolute,
                left: px(0),
                top: px(0),
                width: px(16),
                height: px(Layout::MINI_HEIGHT),
                border_radius: BorderRadius::all(px(3)),
                overflow: Overflow::visible(),
                ..default()
            },
            ZIndex(1),
            Pickable::IGNORE,
        ),
    );
    let material = commands.material(
        entity,
        materials,
        MahjongTileMaterial {
            table_projection: true,
            overlay_projection: false,
            params: Vec4::new(0.0, 1.0, 1.0, if layers == 2 { 2.0 } else { 3.0 }),
            lighting: mahjong_local_light(orientation),
            highlight: Vec4::ZERO,
            elevation: 0.0,
            glyph: assets.visuals.tile_back.clone(),
            height: assets.visuals.tile_back.clone(),
            ..default()
        },
    );
    attach_mahjong_tile_surface(commands, entity, material, true);
}
