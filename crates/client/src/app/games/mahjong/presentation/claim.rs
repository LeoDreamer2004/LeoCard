use super::{
    MAHJONG_CLAIM_FLIGHT_DELAY, MAHJONG_CLAIM_FLIGHT_DURATION, MAHJONG_CLAIM_HAND_SHIFT_DURATION,
    MAHJONG_CLAIM_PRESENTATION_DURATION, MAHJONG_FLOWER_PRESENTATION_DURATION,
    MAHJONG_OWN_HAND_LEFT, MAHJONG_REMOTE_MELD_WIDTH, MahjongAssets, MahjongClaimFlight,
    MahjongClaimHandShift, MahjongClaimHeldTile, MahjongClaimLabel, MahjongClaimPresentationState,
    MahjongFlowerLabel, MahjongTileMaterial, MahjongTileSize, MahjongTileVisual,
    add_mahjong_tile_material, mahjong_claim_landing_time,
};
use crate::app::presentation::{ACCENT, TEXT, add_text, ease_out_cubic, spawn_node};
use crate::app::runtime::UiAssets;
use bevy::prelude::*;
use bevy::ui::FocusPolicy;
use leocard_mahjong::{MahjongClaim, MahjongTileKind};
use leocard_protocol::{MahjongSnapshot, PlayerId};

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

    fn meld_anchor(
        relative: u8,
        meld_count: usize,
        claim: MahjongClaim,
        tile: MahjongTileKind,
    ) -> Vec2 {
        let meld_index = meld_count.saturating_sub(1) as f32;
        let center = match relative {
            0 => Vec2::new(MAHJONG_OWN_HAND_LEFT + 70.0 + meld_index * 140.0, 610.0),
            1 => Vec2::new(1073.5, 516.5 - meld_index * MAHJONG_REMOTE_MELD_WIDTH),
            2 => Vec2::new(827.5 - meld_index * MAHJONG_REMOTE_MELD_WIDTH, 76.5),
            _ => Vec2::new(206.5, 141.5 + meld_index * MAHJONG_REMOTE_MELD_WIDTH),
        };
        let slot = match (claim, tile) {
            (MahjongClaim::Chow { start }, MahjongTileKind::Suited { rank, .. }) => {
                i16::from(rank) - i16::from(start) - 1
            }
            _ => 0,
        } as f32;
        let offset = slot * if relative == 0 { 45.0 } else { 24.0 };
        center
            + match relative {
                0 => Vec2::new(offset, 0.0),
                1 => Vec2::new(0.0, -offset),
                2 => Vec2::new(-offset, 0.0),
                _ => Vec2::new(0.0, offset),
            }
    }

    const fn angle(relative: u8) -> f32 {
        match relative {
            0 => 0.0,
            1 => -std::f32::consts::FRAC_PI_2,
            2 => std::f32::consts::PI,
            _ => std::f32::consts::FRAC_PI_2,
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
    let angle_delta = (flight.target_angle - flight.start_angle + std::f32::consts::PI)
        .rem_euclid(std::f32::consts::TAU)
        - std::f32::consts::PI;
    let angle = flight.start_angle + angle_delta * progress;
    let lift = (raw * std::f32::consts::PI).sin();
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
    let scale = 1.0 + (1.0 - focus) * 0.78 + (focus * std::f32::consts::PI).sin() * 0.06;
    let fade_in = (elapsed / 0.08).clamp(0.0, 1.0);
    let fade_out = ((MAHJONG_CLAIM_PRESENTATION_DURATION - elapsed) / 0.32).clamp(0.0, 1.0);
    (scale, fade_in * fade_out, 7.0 * (1.0 - focus))
}

fn mahjong_flower_label_visual(elapsed: f32) -> (f32, f32, f32) {
    let focus = ease_out_cubic((elapsed / 0.24).clamp(0.0, 1.0));
    let scale = 1.0 + (1.0 - focus) * 0.52 + (focus * std::f32::consts::PI).sin() * 0.04;
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
        let meld_count = game
            .players
            .iter()
            .find(|player| player.id == active.player)
            .map_or(1, |player| player.melds.len());
        let target = MahjongSeatGeometry::meld_anchor(
            target_relative,
            meld_count,
            active.claim,
            tile.kind(),
        );
        let midpoint = (start + target) * 0.5;
        let toward_center = Vec2::new(640.0, 340.0) - midpoint;
        let flight = MahjongClaimFlight {
            player: active.player,
            tile,
            start,
            control: midpoint + toward_center.normalize_or_zero() * 52.0,
            target,
            start_angle: MahjongSeatGeometry::angle(source_relative),
            target_angle: MahjongSeatGeometry::angle(target_relative),
            target_scale: if target_relative == 0 {
                50.0 / 33.0
            } else {
                27.0 / 33.0
            },
        };
        let flight_elapsed = (active.elapsed - MAHJONG_CLAIM_FLIGHT_DELAY).max(0.0);
        let (position, angle, scale, _) = mahjong_claim_flight_pose(flight_elapsed, &flight);
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
                left: px(position.x - 16.5),
                top: px(position.y - 22.5),
                width: px(33),
                min_width: px(33),
                height: px(45),
                overflow: Overflow::visible(),
                ..default()
            },
            UiTransform {
                rotation: Rot2::radians(angle),
                scale: Vec2::splat(scale),
                ..default()
            },
            BoxShadow::new(Color::BLACK.with_alpha(0.38), px(2), px(6), px(0), px(7)),
            ZIndex(90),
            if active.elapsed >= MAHJONG_CLAIM_FLIGHT_DELAY {
                Visibility::Visible
            } else {
                Visibility::Hidden
            },
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

pub(crate) fn animate_mahjong_claim_presentation(
    presentation: Res<MahjongClaimPresentationState>,
    mut materials: ResMut<Assets<MahjongTileMaterial>>,
    mut flights: Query<
        (
            &MahjongClaimFlight,
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
    for (flight, mut node, mut transform, material_node, mut visibility) in &mut flights {
        let Some(active) = active.filter(|active| {
            active.player == flight.player
                && active.tile == Some(flight.tile)
                && (MAHJONG_CLAIM_FLIGHT_DELAY..mahjong_claim_landing_time())
                    .contains(&active.elapsed)
        }) else {
            *visibility = Visibility::Hidden;
            continue;
        };
        let flight_elapsed = active.elapsed - MAHJONG_CLAIM_FLIGHT_DELAY;
        let (position, angle, scale, alpha) = mahjong_claim_flight_pose(flight_elapsed, flight);
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
