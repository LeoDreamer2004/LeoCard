use std::f32::consts;

use super::super::{
    MAHJONG_CLAIM_FLIGHT_DELAY, MAHJONG_CLAIM_FLIGHT_DURATION, MAHJONG_CLAIM_HAND_SHIFT_DURATION,
    MAHJONG_CLAIM_PRESENTATION_DURATION, MAHJONG_FLOWER_PRESENTATION_DURATION, MahjongAssets,
    MahjongClaimFlight, MahjongClaimHandShift, MahjongClaimHeldTile, MahjongClaimLabel,
    MahjongClaimPresentationState, MahjongFlowerLabel, MahjongTableRoot, MahjongTileMaterial,
    mahjong_claim_landing_time,
};

use super::{MahjongTileSize, MahjongTileVisual, add_mahjong_tile_material};
use crate::app::presentation::{ACCENT, TEXT, add_text, ease_out_cubic, spawn_node};
use crate::app::runtime::UiAssets;
use bevy::prelude::*;
use bevy::ui::FocusPolicy;
use leocard_mahjong::MahjongClaim;
use leocard_protocol::{MahjongSnapshot, PlayerId};

#[derive(Component)]
pub(crate) struct MahjongClaimLandingSlot {
    pub player: PlayerId,
}

fn mahjong_claim_label(claim: MahjongClaim) -> &'static str {
    match claim {
        MahjongClaim::Chow { .. } => "吃",
        MahjongClaim::Pung => "碰",
        MahjongClaim::Kong => "杠",
        MahjongClaim::Pass | MahjongClaim::Win => "",
    }
}

#[derive(Clone, Copy)]
pub(super) struct MahjongSeatGeometry {
    own_seat: u8,
}

impl MahjongSeatGeometry {
    pub(super) const fn new(own_seat: u8) -> Self {
        Self { own_seat }
    }

    pub(super) fn relative_player(self, game: &MahjongSnapshot, player: PlayerId) -> Option<u8> {
        game.players
            .iter()
            .find(|candidate| candidate.id == player)
            .map(|candidate| (candidate.seat.0 + 4 - self.own_seat) % 4)
    }

    pub(super) const fn river_anchor(relative: u8) -> Vec2 {
        match relative {
            0 => Vec2::new(640.0, 430.0),
            1 => Vec2::new(805.0, 335.0),
            2 => Vec2::new(640.0, 215.0),
            _ => Vec2::new(475.0, 335.0),
        }
    }

    const fn angle(relative: u8) -> f32 {
        match relative {
            0 => 0.0,
            1 => -consts::FRAC_PI_2,
            2 => consts::PI,
            _ => consts::FRAC_PI_2,
        }
    }
}

fn mahjong_claim_flight_pose(elapsed: f32, flight: &MahjongClaimFlight) -> (Vec2, f32, f32, f32) {
    let raw = (elapsed / MAHJONG_CLAIM_FLIGHT_DURATION).clamp(0.0, 1.0);
    let progress = ease_out_cubic(raw);
    let inverse = 1.0 - progress;
    let position = flight.start * inverse * inverse
        + flight.control * (2.0 * inverse * progress)
        + flight.target * progress * progress;
    let angle_delta = (flight.target_angle - flight.start_angle + consts::PI)
        .rem_euclid(consts::TAU)
        - consts::PI;
    let angle = flight.start_angle + angle_delta * progress;
    let lift = (raw * consts::PI).sin();
    let base_scale = 1.0 + (flight.target_scale - 1.0) * progress;
    (
        position,
        angle,
        base_scale * (1.0 + lift * 0.12),
        (raw / 0.08).min(1.0),
    )
}

fn mahjong_claim_label_visual(elapsed: f32) -> (f32, f32, f32) {
    let focus = (elapsed / 0.24).clamp(0.0, 1.0);
    let focus = ease_out_cubic(focus);
    let scale = 1.0 + (1.0 - focus) * 0.78 + (focus * consts::PI).sin() * 0.06;
    let fade_in = (elapsed / 0.08).clamp(0.0, 1.0);
    let fade_out = ((MAHJONG_CLAIM_PRESENTATION_DURATION - elapsed) / 0.32).clamp(0.0, 1.0);
    (scale, fade_in * fade_out, 7.0 * (1.0 - focus))
}

fn mahjong_flower_label_visual(elapsed: f32) -> (f32, f32, f32) {
    let focus = ease_out_cubic((elapsed / 0.24).clamp(0.0, 1.0));
    let scale = 1.0 + (1.0 - focus) * 0.52 + (focus * consts::PI).sin() * 0.04;
    let fade_in = (elapsed / 0.08).clamp(0.0, 1.0);
    let fade_out = ((MAHJONG_FLOWER_PRESENTATION_DURATION - elapsed) / 0.28).clamp(0.0, 1.0);
    (scale, fade_in * fade_out, 6.0 * (1.0 - focus))
}

pub(crate) fn mahjong_claim_hand_shift_x(elapsed: f32, distance: f32) -> f32 {
    let progress = ease_out_cubic((elapsed / MAHJONG_CLAIM_HAND_SHIFT_DURATION).clamp(0.0, 1.0));
    -distance * (1.0 - progress)
}

pub(super) fn mahjong_claim_held_tile_visual(elapsed: f32) -> (f32, f32) {
    let progress = ease_out_cubic((elapsed / 0.24).clamp(0.0, 1.0));
    (0.06 + progress * 0.94, 5.0 * (1.0 - progress))
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn render_mahjong_claim_presentation(
    commands: &mut Commands,
    table: Entity,
    game: &MahjongSnapshot,
    own_seat: u8,
    presentation: &MahjongClaimPresentationState,
    assets: &UiAssets,
    game_assets: &MahjongAssets,
    materials: &mut Assets<MahjongTileMaterial>,
) {
    let geometry = MahjongSeatGeometry::new(own_seat);
    let Some(active) = presentation
        .active
        .as_ref()
        .filter(|active| active.match_id == game.match_id)
    else {
        return;
    };
    let Some(target_relative) = geometry.relative_player(game, active.player) else {
        return;
    };
    if let (Some(source), Some(tile)) = (active.source, active.tile)
        && active.elapsed < mahjong_claim_landing_time()
    {
        let Some(source_relative) = geometry.relative_player(game, source) else {
            return;
        };
        let start = MahjongSeatGeometry::river_anchor(source_relative);
        let flight = MahjongClaimFlight {
            player: active.player,
            tile,
            start,
            control: start,
            target: start,
            start_angle: MahjongSeatGeometry::angle(source_relative),
            target_angle: MahjongSeatGeometry::angle(target_relative),
            target_scale: if target_relative == 0 {
                56.0 / 33.0
            } else {
                27.0 / 33.0
            },
        };
        let tile = add_mahjong_tile_material(
            commands,
            table,
            MahjongTileVisual {
                kind: Some(tile.kind()),
                size: MahjongTileSize::River,
                index: 0,
                highlighted: false,
                deal: None,
                relative: target_relative,
            },
            game_assets,
            materials,
        );
        commands.entity(tile).insert((
            flight,
            Node {
                position_type: PositionType::Absolute,
                left: px(start.x - 16.5),
                top: px(start.y - 22.5),
                width: px(33),
                min_width: px(33),
                height: px(45),
                overflow: Overflow::visible(),
                ..default()
            },
            UiTransform {
                rotation: Rot2::radians(MahjongSeatGeometry::angle(source_relative)),
                ..default()
            },
            BoxShadow::new(Color::BLACK.with_alpha(0.38), px(2), px(6), px(0), px(7)),
            ZIndex(90),
            Visibility::Hidden,
        ));
    }

    let label_position = MahjongSeatGeometry::river_anchor(target_relative);
    let (scale, alpha, offset_y) = mahjong_claim_label_visual(active.elapsed);
    let holder = spawn_node(
        commands,
        table,
        Node {
            position_type: PositionType::Absolute,
            left: px(label_position.x - 55.0),
            top: px(label_position.y - 34.0),
            width: px(110),
            height: px(68),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            ..default()
        },
        None,
    );
    let text = add_text(
        commands,
        holder,
        mahjong_claim_label(active.claim),
        42.0,
        ACCENT.with_alpha(alpha),
        assets,
    );
    commands.entity(text).insert(TextShadow {
        offset: Vec2::new(1.5, 3.0),
        color: Color::BLACK.with_alpha(0.72 * alpha),
    });
    commands.entity(holder).insert((
        MahjongClaimLabel {
            player: active.player,
            text,
        },
        UiTransform {
            translation: Val2::px(0.0, offset_y),
            scale: Vec2::splat(scale),
            ..default()
        },
        ZIndex(89),
        FocusPolicy::Pass,
    ));
}

pub(crate) fn render_mahjong_flower_presentations(
    commands: &mut Commands,
    table: Entity,
    game: &MahjongSnapshot,
    own_seat: u8,
    presentation: &MahjongClaimPresentationState,
    assets: &UiAssets,
) {
    let geometry = MahjongSeatGeometry::new(own_seat);
    for active in presentation
        .flowers
        .iter()
        .filter(|active| active.match_id == game.match_id)
    {
        let Some(relative) = geometry.relative_player(game, active.player) else {
            continue;
        };
        let position = MahjongSeatGeometry::river_anchor(relative);
        let (scale, alpha, offset_y) = mahjong_flower_label_visual(active.elapsed);
        let holder = spawn_node(
            commands,
            table,
            Node {
                position_type: PositionType::Absolute,
                left: px(position.x - 60.0),
                top: px(position.y - 34.0),
                width: px(120),
                height: px(68),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
            None,
        );
        let text = add_text(
            commands,
            holder,
            "补花",
            38.0,
            TEXT.with_alpha(alpha),
            assets,
        );
        commands.entity(text).insert(TextShadow {
            offset: Vec2::new(1.5, 3.0),
            color: Color::BLACK.with_alpha(0.72 * alpha),
        });
        commands.entity(holder).insert((
            MahjongFlowerLabel {
                player: active.player,
                text,
            },
            UiTransform {
                translation: Val2::px(0.0, offset_y),
                scale: Vec2::splat(scale),
                ..default()
            },
            ZIndex(89),
            FocusPolicy::Pass,
        ));
    }
}

pub(crate) fn animate_mahjong_flower_presentations(
    presentation: Res<MahjongClaimPresentationState>,
    mut labels: Query<(&MahjongFlowerLabel, &mut UiTransform, &mut Visibility)>,
    mut texts: Query<(&mut TextColor, &mut TextShadow)>,
) {
    for (label, mut transform, mut visibility) in &mut labels {
        let Some(active) = presentation
            .flowers
            .iter()
            .find(|active| active.player == label.player)
        else {
            *visibility = Visibility::Hidden;
            continue;
        };
        let (scale, alpha, offset_y) = mahjong_flower_label_visual(active.elapsed);
        transform.translation = Val2::px(0.0, offset_y);
        transform.scale = Vec2::splat(scale);
        if let Ok((mut color, mut shadow)) = texts.get_mut(label.text) {
            color.0 = TEXT.with_alpha(alpha);
            shadow.color = Color::BLACK.with_alpha(0.72 * alpha);
        }
        *visibility = if alpha > 0.0 {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }
}

#[expect(
    clippy::type_complexity,
    reason = "disjoint Bevy queries encode mutually exclusive presentation components"
)]
#[expect(
    clippy::too_many_arguments,
    reason = "Bevy needs separate disjoint queries for the flight, label, and hand animations"
)]
pub(crate) fn animate_mahjong_claim_presentation(
    presentation: Res<MahjongClaimPresentationState>,
    mut materials: ResMut<Assets<MahjongTileMaterial>>,
    tables: Query<(&ComputedNode, &UiGlobalTransform), With<MahjongTableRoot>>,
    slots: Query<(&MahjongClaimLandingSlot, &ComputedNode, &UiGlobalTransform)>,
    mut flights: Query<
        (
            &mut MahjongClaimFlight,
            &mut Node,
            &mut UiTransform,
            &MaterialNode<MahjongTileMaterial>,
            &mut Visibility,
        ),
        (
            Without<MahjongClaimLabel>,
            Without<MahjongClaimHandShift>,
            Without<MahjongClaimHeldTile>,
        ),
    >,
    mut labels: Query<
        (&MahjongClaimLabel, &mut UiTransform, &mut Visibility),
        (
            Without<MahjongClaimFlight>,
            Without<MahjongClaimHandShift>,
            Without<MahjongClaimHeldTile>,
        ),
    >,
    mut hands: Query<
        (&MahjongClaimHandShift, &mut UiTransform),
        (
            Without<MahjongClaimFlight>,
            Without<MahjongClaimLabel>,
            Without<MahjongClaimHeldTile>,
        ),
    >,
    mut held_tiles: Query<
        (&MahjongClaimHeldTile, &mut UiTransform, &mut Visibility),
        (
            Without<MahjongClaimFlight>,
            Without<MahjongClaimLabel>,
            Without<MahjongClaimHandShift>,
        ),
    >,
    mut texts: Query<(&mut TextColor, &mut TextShadow)>,
) {
    let active = presentation.active.as_ref();
    for (mut flight, mut node, mut transform, material_node, mut visibility) in &mut flights {
        let Some(active) = active.filter(|active| {
            active.player == flight.player
                && active.tile == Some(flight.tile)
                && (MAHJONG_CLAIM_FLIGHT_DELAY..mahjong_claim_landing_time())
                    .contains(&active.elapsed)
        }) else {
            *visibility = Visibility::Hidden;
            continue;
        };
        let target = tables
            .single()
            .ok()
            .and_then(|(table_node, table_transform)| {
                let (_, slot_node, slot_transform) = slots
                    .iter()
                    .find(|(slot, _, _)| slot.player == flight.player)?;
                if table_node.size().min_element() <= 1.0 || slot_node.size().min_element() <= 1.0 {
                    return None;
                }
                let inverse = table_transform.try_inverse()?;
                let target = (inverse
                    .transform_point2(slot_transform.to_scale_angle_translation().2)
                    + table_node.size() * 0.5)
                    * table_node.inverse_scale_factor();
                let center = table_node.size() * 0.5 * table_node.inverse_scale_factor();
                Some((target, center))
            });
        let Some((target, center)) = target else {
            *visibility = Visibility::Hidden;
            continue;
        };
        flight.target = target;
        let midpoint = (flight.start + target) * 0.5;
        flight.control = midpoint + (center - midpoint).normalize_or_zero() * 52.0;
        let flight_elapsed = active.elapsed - MAHJONG_CLAIM_FLIGHT_DELAY;
        let (position, angle, scale, alpha) = mahjong_claim_flight_pose(flight_elapsed, &flight);
        node.left = px(position.x - 16.5);
        node.top = px(position.y - 22.5);
        transform.rotation = Rot2::radians(angle);
        transform.scale = Vec2::splat(scale);
        if let Some(mut material) = materials.get_mut(&material_node.0) {
            material.params.z = alpha;
        }
        *visibility = Visibility::Visible;
    }
    for (label, mut transform, mut visibility) in &mut labels {
        let Some(active) = active.filter(|active| active.player == label.player) else {
            *visibility = Visibility::Hidden;
            continue;
        };
        let (scale, alpha, offset_y) = mahjong_claim_label_visual(active.elapsed);
        transform.translation = Val2::px(0.0, offset_y);
        transform.scale = Vec2::splat(scale);
        if let Ok((mut color, mut shadow)) = texts.get_mut(label.text) {
            color.0 = ACCENT.with_alpha(alpha);
            shadow.color = Color::BLACK.with_alpha(0.72 * alpha);
        }
        *visibility = if alpha > 0.0 {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }
    for (hand, mut transform) in &mut hands {
        let elapsed = active
            .filter(|active| active.player == hand.player && active.shift_hand)
            .map_or(MAHJONG_CLAIM_HAND_SHIFT_DURATION, |active| active.elapsed);
        transform.translation = Val2::px(mahjong_claim_hand_shift_x(elapsed, hand.distance), 0.0);
    }
    for (tile, mut transform, mut visibility) in &mut held_tiles {
        let Some(active) = active.filter(|active| {
            active.player == tile.player
                && active.source.is_some()
                && active.elapsed < mahjong_claim_landing_time()
        }) else {
            *visibility = Visibility::Hidden;
            continue;
        };
        let (scale_x, offset_y) = mahjong_claim_held_tile_visual(active.elapsed);
        transform.translation = Val2::px(0.0, offset_y);
        transform.scale = Vec2::new(scale_x, 1.0);
        *visibility = Visibility::Visible;
    }
}
