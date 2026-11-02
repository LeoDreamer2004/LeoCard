//! The measured table camera is projected directly into UI quads.
//! Calibration and pixel measurements: docs/mahjong-perspective.md.

use super::layout::MahjongLayout as Layout;
use super::{MahjongResponseTile, MahjongTableRoot, MahjongTileMaterial, MahjongTileSurface};
use bevy::picking::Pickable;
use bevy::prelude::*;
use leocard_ui::{UiComposer, spawn_node};

pub(super) const WORLD_SCALE: f32 = 16.0;
pub(super) const WORLD_ORIGIN: Vec2 = Vec2::new(640.0, 355.0);
pub(super) const REFERENCE_SIZE: Vec2 = Vec2::new(2204.0, 1217.0);
const CAMERA_POSITION: Vec3 = Vec3::new(0.0, 27.708_225, -29.475_252);
const CAMERA_PITCH: f32 = 0.809_956_25;
const FOCAL_LENGTH: f32 = 2_491.018_3;
const PRINCIPAL_POINT: Vec2 = Vec2::new(1095.0, 608.5);

pub(super) fn world_layout(position: Vec2) -> Vec2 {
    WORLD_ORIGIN + Vec2::new(position.x, -position.y) * WORLD_SCALE
}

pub(super) fn project_reference(position: Vec3) -> Vec2 {
    let relative = position - CAMERA_POSITION;
    let (sine, cosine) = CAMERA_PITCH.sin_cos();
    let depth = -relative.y * sine + relative.z * cosine;
    PRINCIPAL_POINT
        + Vec2::new(relative.x, -relative.y * cosine - relative.z * sine) * (FOCAL_LENGTH / depth)
}

pub(super) fn central_screen_offset() -> Vec2 {
    (project_reference(Vec3::new(0.0, 0.0, -Layout::CENTER_FORWARD))
        - project_reference(Vec3::ZERO))
        * (1280.0 / REFERENCE_SIZE.x)
}

fn local_vector(vector: Vec3, angle: f32) -> Vec3 {
    let (sine, cosine) = angle.sin_cos();
    Vec3::new(
        cosine * vector.x - sine * vector.z,
        vector.y,
        sine * vector.x + cosine * vector.z,
    )
}

fn world_vector(vector: Vec3, angle: f32) -> Vec3 {
    local_vector(vector, -angle)
}

pub(super) fn sync_mahjong_tile_projection(
    tiles: Query<(
        Entity,
        &ComputedNode,
        &UiGlobalTransform,
        &MahjongTileSurface,
        Option<&MahjongResponseTile>,
    )>,
    parents: Query<&ChildOf>,
    tables: Query<(&ComputedNode, &UiGlobalTransform), With<MahjongTableRoot>>,
    viewport: Res<MahjongProjectionViewport>,
    mut materials: ResMut<Assets<MahjongTileMaterial>>,
) {
    for (entity, node, transform, surface, response) in &tiles {
        let Some(original) = materials.get(&surface.0) else {
            continue;
        };
        let mut material = original.clone();
        material.ray_origin.w = 0.0;
        // Own hands, guides and settlement rows retain their front-facing UI presentation.
        if !material.table_projection {
            continue;
        }
        let mut ancestor = entity;
        let mut table = None;
        while let Ok(parent) = parents.get(ancestor) {
            ancestor = parent.parent();
            if let Ok(found) = tables.get(ancestor) {
                table = Some(found);
                break;
            }
        }
        let Some((table_node, table_transform)) = table else {
            continue;
        };
        let Some(inverse) = table_transform.try_inverse() else {
            continue;
        };
        let tile_center = transform.to_scale_angle_translation().2;
        let inverse_ui_scale = table_node.inverse_scale_factor();
        let layout_center =
            (inverse.transform_point2(tile_center) + table_node.size() * 0.5) * inverse_ui_scale;
        let world = if material.overlay_projection {
            let ground = ground_from_layout(layout_center, viewport.size);
            Vec2::new(ground.x, -ground.y)
        } else {
            (layout_center - WORLD_ORIGIN) / WORLD_SCALE
        };
        let transform_scale = table_transform.matrix2.x_axis.length();
        let table_scale = transform_scale / inverse_ui_scale;
        let local_axis = inverse.transform_vector2(transform.matrix2.x_axis);
        let angle = local_axis.y.atan2(local_axis.x);
        let tile_scale = local_axis.length()
            * if material.overlay_projection {
                overlay_tile_scale(
                    layout_center,
                    viewport.size,
                    node.size().x * node.inverse_scale_factor(),
                )
            } else {
                1.0
            };
        if tile_scale < 0.0001 {
            if material != *original {
                *materials
                    .get_mut(&surface.0)
                    .expect("retained tile material") = material;
            }
            continue;
        }
        let upright_side = (material.params.w + 2.0).abs() < 0.1;
        let upright_opposite = (material.params.w + 3.0).abs() < 0.1;
        let wall = material.params.w > 0.5;
        let half = if upright_side {
            Vec3::new(0.5, 0.60, 0.325)
        } else if upright_opposite {
            Vec3::new(0.5, 0.615, 0.28)
        } else if wall {
            Vec3::new(
                0.5,
                Layout::TABLE_TILE_THICKNESS
                    * if (material.params.w - 2.0).abs() < 0.1 {
                        1.0
                    } else {
                        0.5
                    },
                0.5 / Layout::FACE_ASPECT_RATIO,
            )
        } else {
            Vec3::new(
                0.5,
                Layout::TABLE_TILE_THICKNESS * 0.5,
                0.5 / Layout::FACE_ASPECT_RATIO,
            )
        } * tile_scale;
        let elevation = material.elevation / 30.0 * tile_scale;
        let center = Vec3::new(world.x, half.y + elevation, -world.y);
        // The eye is centered: equal-depth tiles nearer the center must cover their outer neighbors.
        // Camera distance also puts the nearer row and the upper tile of a kong in front.
        let eye_distance = center.distance(CAMERA_POSITION);
        material.paint_order = (100_000.0 - eye_distance * 1000.0).round() as i32;
        let world_light = Vec3::new(-0.32, 2.3, 0.45);
        let mut min = Vec2::splat(f32::INFINITY);
        let mut max = Vec2::splat(f32::NEG_INFINITY);
        let mut box_min = min;
        let mut box_max = max;
        for x in [-1.0, 1.0] {
            for y in [-1.0, 1.0] {
                for z in [-1.0, 1.0] {
                    let vertex = center + world_vector(half * Vec3::new(x, y, z), angle);
                    let projected = project_reference(vertex);
                    box_min = box_min.min(projected);
                    box_max = box_max.max(projected);
                    min = min.min(projected);
                    max = max.max(projected);
                    let ground = vertex - world_light * (vertex.y / world_light.y);
                    let shadow = project_reference(ground);
                    min = min.min(shadow);
                    max = max.max(shadow);
                }
            }
        }
        let (scene_scale, _) = camera_canvas(viewport.size);
        let arrow_scale = 1280.0 / REFERENCE_SIZE.x * scene_scale;
        let arrow_tip = Vec2::new((box_min.x + box_max.x) * 0.5, box_min.y - 6.0 / arrow_scale);
        if response.is_some() {
            min = min.min(arrow_tip - Vec2::new(10.0, 20.0) / arrow_scale);
            max = max.max(arrow_tip + Vec2::new(10.0, 0.0) / arrow_scale);
        }
        min -= Vec2::splat(10.0);
        max += Vec2::splat(10.0);
        let (camera_scale, canvas_origin) = camera_canvas(viewport.size);
        let reference_scale = 1280.0 / REFERENCE_SIZE.x * camera_scale;
        let table_top_left = table_transform.to_scale_angle_translation().2
            - table_node.size() * transform_scale * 0.5;
        material.paint_bounds = (table_top_left
            + (canvas_origin + min * reference_scale) * table_scale)
            .extend((max.x - min.x) * reference_scale * table_scale)
            .extend((max.y - min.y) * reference_scale * table_scale);
        material.response = if response.is_some() {
            ((arrow_tip - min) / (max - min))
                .extend(10.0 / arrow_scale / (max.x - min.x))
                .extend(14.0 / arrow_scale / (max.y - min.y))
        } else {
            Vec4::ZERO
        };
        let ray_center = (min + max) * 0.5;
        let (sine, cosine) = CAMERA_PITCH.sin_cos();
        let forward = Vec3::new(0.0, -sine, cosine);
        let down = Vec3::new(0.0, -cosine, -sine);
        let ray = forward
            + Vec3::X * ((ray_center.x - PRINCIPAL_POINT.x) / FOCAL_LENGTH)
            + down * ((ray_center.y - PRINCIPAL_POINT.y) / FOCAL_LENGTH);
        material.ray_origin = local_vector(CAMERA_POSITION - center, angle).extend(center.y);
        material.ray_forward = local_vector(ray, angle).extend(0.0);
        material.ray_right =
            local_vector(Vec3::X * ((max.x - min.x) / FOCAL_LENGTH), angle).extend(0.0);
        material.ray_down =
            local_vector(down * ((max.y - min.y) / FOCAL_LENGTH), angle).extend(0.0);
        let bevel = if upright_side || upright_opposite || wall {
            0.045
        } else {
            0.070
        };
        material.box_half_size = half.extend(bevel * tile_scale);
        let local_light = local_vector(world_light, angle);
        material.lighting.x = local_light.x;
        material.lighting.y = local_light.z;
        if material != *original {
            *materials
                .get_mut(&surface.0)
                .expect("retained tile material") = material;
        }
    }
}

pub(super) const REFERENCE_DESIGN_HEIGHT: f32 = 1217.0 * 1280.0 / 2204.0;

pub(super) fn camera_canvas(viewport: Vec2) -> (f32, Vec2) {
    let scale = (viewport.y / REFERENCE_DESIGN_HEIGHT).min(viewport.x / 1280.0);
    (scale, Vec2::new(640.0 * (1.0 - scale), 0.0))
}

pub(super) fn project_layout(position: Vec3, viewport: Vec2) -> Vec2 {
    let (scale, origin) = camera_canvas(viewport);
    origin + project_reference(position) * (1280.0 / REFERENCE_SIZE.x * scale)
}

pub(super) fn project_logical(position: Vec2) -> Vec2 {
    let point = (position - WORLD_ORIGIN) / WORLD_SCALE;
    project_layout(
        Vec3::new(point.x, 0.0, -point.y),
        Vec2::new(1280.0, REFERENCE_DESIGN_HEIGHT),
    )
}

pub(super) fn ground_from_layout(position: Vec2, viewport: Vec2) -> Vec2 {
    let (scale, origin) = camera_canvas(viewport);
    let pixel = (position - origin) / (1280.0 / REFERENCE_SIZE.x * scale);
    let image = (pixel - PRINCIPAL_POINT) / FOCAL_LENGTH;
    let (sine, cosine) = CAMERA_PITCH.sin_cos();
    let ray = Vec3::new(image.x, -sine - image.y * cosine, cosine - image.y * sine);
    let ground = CAMERA_POSITION + ray * (-CAMERA_POSITION.y / ray.y);
    Vec2::new(ground.x, ground.z)
}

pub(super) fn overlay_tile_scale(position: Vec2, viewport: Vec2, width: f32) -> f32 {
    let ground = ground_from_layout(position, viewport);
    let unit = project_layout(Vec3::new(ground.x + 0.5, 0.0, ground.y), viewport).x
        - project_layout(Vec3::new(ground.x - 0.5, 0.0, ground.y), viewport).x;
    width / unit
}

#[derive(Component)]
pub(super) struct MahjongProjectionFrame {
    pub center: Vec2,
    pub anchor_bottom: bool,
}

pub(super) fn projected_overlay(commands: &mut UiComposer, parent: Entity) -> Entity {
    canvas_overlay(commands, parent, false)
}

pub(super) fn own_overlay(commands: &mut UiComposer, parent: Entity) -> Entity {
    canvas_overlay(commands, parent, true)
}

fn canvas_overlay(commands: &mut UiComposer, parent: Entity, anchor_bottom: bool) -> Entity {
    let overlay = spawn_node(
        commands,
        parent,
        Node {
            position_type: PositionType::Absolute,
            left: px(0),
            top: px(0),
            width: px(1280),
            height: px(REFERENCE_DESIGN_HEIGHT),
            ..default()
        },
        None,
    );
    commands.entity(overlay).insert((
        MahjongProjectionFrame {
            center: Vec2::new(640.0, REFERENCE_DESIGN_HEIGHT * 0.5),
            anchor_bottom,
        },
        UiTransform::default(),
        ZIndex(95),
        Pickable::IGNORE,
    ));
    overlay
}

pub(super) fn sync_mahjong_projection_frames(
    viewport: Res<MahjongProjectionViewport>,
    mut frames: Query<(&MahjongProjectionFrame, &mut UiTransform)>,
) {
    for (frame, mut transform) in &mut frames {
        let (scale, mut origin) = camera_canvas(viewport.size);
        if frame.anchor_bottom {
            origin.y += viewport.size.y - REFERENCE_DESIGN_HEIGHT * scale;
        }
        let offset = origin + frame.center * (scale - 1.0);
        let next = UiTransform {
            scale: Vec2::splat(scale),
            translation: Val2::px(offset.x, offset.y),
            ..default()
        };
        if *transform != next {
            *transform = next;
        }
    }
}

#[derive(Resource)]
pub(crate) struct MahjongProjectionViewport {
    pub size: Vec2,
}

impl Default for MahjongProjectionViewport {
    fn default() -> Self {
        Self {
            size: Vec2::new(1280.0, REFERENCE_DESIGN_HEIGHT),
        }
    }
}

pub(super) fn sync_mahjong_projection_viewport(
    tables: Query<&ChildOf, With<MahjongTableRoot>>,
    nodes: Query<&ComputedNode>,
    mut viewport: ResMut<MahjongProjectionViewport>,
) {
    let Some(size) = tables
        .iter()
        .filter_map(|parent| nodes.get(parent.parent()).ok())
        .map(|node| node.size() * node.inverse_scale_factor())
        .find(|size| size.min_element() > 1.0)
    else {
        return;
    };
    if viewport.size != size {
        viewport.size = size;
    }
}
