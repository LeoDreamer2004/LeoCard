use super::*;
use leocard_protocol::{MahjongHandResultView, MahjongWinView};
use leocard_protocol::{MahjongPhaseView, MahjongSnapshot};

const MAJOR_FAN_GLYPH_DELAY: f32 = 1.02;
const MAJOR_FAN_GLYPH_INTERVAL: f32 = 0.30;

pub(super) fn mahjong_win_effect_color(
    tier: MahjongWinEffectTier,
    secondary: bool,
    alpha: f32,
) -> Color {
    let color = match (tier, secondary) {
        (MahjongWinEffectTier::Normal, false) => Color::srgb(0.76, 1.0, 0.84),
        (MahjongWinEffectTier::Normal, true) => Color::srgb(0.35, 0.82, 0.58),
        (MahjongWinEffectTier::HighTotal, false) => Color::srgb(0.30, 1.0, 0.68),
        (MahjongWinEffectTier::HighTotal, true) => Color::srgb(0.58, 0.94, 0.84),
        (MahjongWinEffectTier::MajorFan, false) => Color::srgb(1.0, 0.76, 0.18),
        (MahjongWinEffectTier::MajorFan, true) => Color::srgb(1.0, 0.95, 0.68),
    };
    color.with_alpha(alpha)
}

pub(super) fn mahjong_win_effect_elapsed(
    summary_elapsed: f32,
    reveal_duration: f32,
    tier: MahjongWinEffectTier,
) -> Option<f32> {
    let elapsed = summary_elapsed + reveal_duration;
    (0.0..tier.duration()).contains(&elapsed).then_some(elapsed)
}

pub(super) fn mahjong_win_effect_visual(
    summary_elapsed: f32,
    reveal_duration: f32,
    tier: MahjongWinEffectTier,
) -> Option<(f32, f32, f32)> {
    let elapsed = mahjong_win_effect_elapsed(summary_elapsed, reveal_duration, tier)?;
    let duration = tier.duration();
    let focus = ease_out_cubic((elapsed / 0.26).clamp(0.0, 1.0));
    let impact = match tier {
        MahjongWinEffectTier::Normal => 0.72,
        MahjongWinEffectTier::HighTotal => 1.02,
        MahjongWinEffectTier::MajorFan => 1.28,
    };
    let scale = 1.0 + (1.0 - focus) * impact + (focus * std::f32::consts::PI).sin() * 0.08;
    let fade_in = (elapsed / 0.07).clamp(0.0, 1.0);
    let fade_out = ((duration - elapsed) / 0.26).clamp(0.0, 1.0);
    Some((scale, fade_in * fade_out, 9.0 * (1.0 - focus)))
}

pub fn mahjong_major_fan_impact_times(result: &MahjongHandResultView) -> Vec<f32> {
    let mut impacts = Vec::new();
    for (winner_index, winner) in result.winners.iter().enumerate() {
        if mahjong_win_effect_tier(winner) != MahjongWinEffectTier::MajorFan {
            continue;
        }
        let Some(major_fan) = winner.score.fans.iter().max_by_key(|fan| fan.fan.points()) else {
            continue;
        };
        let start = mahjong_win_stage_start(result, winner_index);
        for glyph_index in 0..major_fan.fan.name().chars().count() {
            impacts.push(
                start + MAJOR_FAN_GLYPH_DELAY + glyph_index as f32 * MAJOR_FAN_GLYPH_INTERVAL,
            );
        }
    }
    impacts
}

fn spawn_mahjong_win_decoration(
    commands: &mut Commands,
    parent: Entity,
    node: Node,
    tier: MahjongWinEffectTier,
    reveal_duration: f32,
    kind: MahjongWinDecorationKind,
    z_index: i32,
) {
    let decoration = spawn_node(commands, parent, node, Some(Color::NONE));
    commands.entity(decoration).insert((
        MahjongWinDecoration {
            tier,
            reveal_duration,
            kind,
        },
        UiTransform::IDENTITY,
        BorderColor::all(Color::NONE),
        ZIndex(z_index),
        FocusPolicy::Pass,
    ));
}

fn add_mahjong_win_aura(
    commands: &mut Commands,
    holder: Entity,
    tier: MahjongWinEffectTier,
    reveal_duration: f32,
) {
    spawn_mahjong_win_decoration(
        commands,
        holder,
        Node {
            position_type: PositionType::Absolute,
            left: px(20),
            top: px(8),
            width: px(70),
            height: px(70),
            border_radius: BorderRadius::all(percent(50)),
            ..default()
        },
        tier,
        reveal_duration,
        MahjongWinDecorationKind::Halo,
        -3,
    );

    let ring_count = match tier {
        MahjongWinEffectTier::Normal => 1,
        MahjongWinEffectTier::HighTotal => 2,
        MahjongWinEffectTier::MajorFan => 3,
    };
    for index in 0..ring_count {
        let size = 68.0 + index as f32 * 7.0;
        spawn_mahjong_win_decoration(
            commands,
            holder,
            Node {
                position_type: PositionType::Absolute,
                left: px((110.0 - size) * 0.5),
                top: px((86.0 - size) * 0.5),
                width: px(size),
                height: px(size),
                border: UiRect::all(px(if index == 0 { 2.5 } else { 1.5 })),
                border_radius: BorderRadius::all(percent(50)),
                ..default()
            },
            tier,
            reveal_duration,
            MahjongWinDecorationKind::Ring {
                delay: index as f32 * 0.055,
                start_scale: 0.54 + index as f32 * 0.08,
                end_scale: 1.40 + index as f32 * 0.14,
                max_alpha: 0.72 - index as f32 * 0.10,
            },
            -2,
        );
    }

    let ray_count = match tier {
        MahjongWinEffectTier::Normal => 0,
        MahjongWinEffectTier::HighTotal => 6,
        MahjongWinEffectTier::MajorFan => 12,
    };
    for index in 0..ray_count {
        let angle = index as f32 / ray_count as f32 * std::f32::consts::TAU
            + std::f32::consts::FRAC_PI_4 / 2.0;
        let direction = Vec2::new(angle.cos(), angle.sin());
        let width = if tier == MahjongWinEffectTier::MajorFan && index % 3 == 0 {
            18.0
        } else {
            12.0
        };
        spawn_mahjong_win_decoration(
            commands,
            holder,
            Node {
                position_type: PositionType::Absolute,
                left: px(55.0 - width * 0.5),
                top: px(41.5),
                width: px(width),
                height: px(if index % 2 == 0 { 3.5 } else { 2.5 }),
                border_radius: BorderRadius::all(percent(50)),
                ..default()
            },
            tier,
            reveal_duration,
            MahjongWinDecorationKind::Ray {
                direction,
                distance: 42.0 + (index % 3) as f32 * 8.0,
                delay: 0.06 + (index % 4) as f32 * 0.018,
                secondary: index % 2 == 1,
            },
            -1,
        );
    }
}

pub fn render_mahjong_win_effects(
    commands: &mut Commands,
    table: Entity,
    game: &MahjongSnapshot,
    own_seat: u8,
    animation: &GameSummaryAnimation,
    assets: &UiAssets,
    materials: &mut Assets<MahjongTileMaterial>,
) {
    let geometry = MahjongSeatGeometry::new(own_seat);
    let MahjongPhaseView::Finished { result } = &game.phase else {
        return;
    };
    let reveal_duration = mahjong_win_reveal_duration(result);
    for (winner_index, winner) in result.winners.iter().enumerate() {
        let tier = mahjong_win_effect_tier(winner);
        render_mahjong_win_stage(
            commands,
            table,
            game,
            own_seat,
            result,
            winner_index,
            reveal_duration,
            assets,
            materials,
        );
        let Some((scale, alpha, offset_y)) =
            mahjong_win_effect_visual(animation.elapsed, reveal_duration, tier)
        else {
            continue;
        };
        let Some(relative) = geometry.relative_player(game, winner.player) else {
            continue;
        };
        let position = MahjongSeatGeometry::river_anchor(relative);
        let holder = spawn_node(
            commands,
            table,
            Node {
                position_type: PositionType::Absolute,
                left: px(position.x - 55.0),
                top: px(position.y - 43.0),
                width: px(110),
                height: px(86),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                overflow: Overflow::visible(),
                ..default()
            },
            None,
        );
        add_mahjong_win_aura(commands, holder, tier, reveal_duration);
        let self_draw = winner.from.is_none();
        let text = add_text(
            commands,
            holder,
            if self_draw { "自摸" } else { "和" },
            if self_draw {
                match tier {
                    MahjongWinEffectTier::Normal => 42.0,
                    MahjongWinEffectTier::HighTotal => 46.0,
                    MahjongWinEffectTier::MajorFan => 50.0,
                }
            } else {
                match tier {
                    MahjongWinEffectTier::Normal => 54.0,
                    MahjongWinEffectTier::HighTotal => 60.0,
                    MahjongWinEffectTier::MajorFan => 66.0,
                }
            },
            mahjong_win_effect_color(tier, false, alpha),
            assets,
        );
        commands.entity(text).insert((
            MahjongWinEffectText {
                tier,
                reveal_duration,
            },
            TextShadow {
                offset: Vec2::new(2.0, 4.0),
                color: Color::BLACK.with_alpha(0.76 * alpha),
            },
            ZIndex(1),
        ));
        commands.entity(holder).insert((
            MahjongWinEffect {
                tier,
                reveal_duration,
            },
            UiTransform {
                translation: Val2::px(0.0, offset_y),
                scale: Vec2::splat(scale),
                ..default()
            },
            ZIndex(95),
            FocusPolicy::Pass,
        ));
    }
}

fn mahjong_winning_tile_anchor(
    game: &MahjongSnapshot,
    own_seat: u8,
    winner: &MahjongWinView,
) -> Option<Vec2> {
    let geometry = MahjongSeatGeometry::new(own_seat);
    if let Some(source) = winner.from {
        return geometry
            .relative_player(game, source)
            .map(MahjongSeatGeometry::river_anchor);
    }
    geometry
        .relative_player(game, winner.player)
        .map(|relative| match relative {
            0 => Vec2::new(1005.0, 642.0),
            1 => Vec2::new(1080.0, 450.0),
            2 => Vec2::new(445.0, 92.0),
            _ => Vec2::new(200.0, 220.0),
        })
}

fn add_mahjong_win_stage_component(
    commands: &mut Commands,
    entity: Entity,
    tier: MahjongWinEffectTier,
    reveal_duration: f32,
    start: f32,
    duration: f32,
    kind: MahjongWinStageKind,
    z_index: i32,
) {
    commands.entity(entity).insert((
        MahjongWinStagePart {
            tier,
            reveal_duration,
            start,
            duration,
            kind,
        },
        UiTransform::IDENTITY,
        Visibility::Hidden,
        ZIndex(z_index),
        FocusPolicy::Pass,
    ));
}

fn add_mahjong_high_focus_rays(
    commands: &mut Commands,
    table: Entity,
    reveal_duration: f32,
    start: f32,
    duration: f32,
) {
    const RAY_COUNT: usize = 30;
    let center = Vec2::new(DESIGN_WIDTH * 0.5, 340.0);
    for index in 0..RAY_COUNT {
        let phase = index as f32 / RAY_COUNT as f32;
        let angle = phase * std::f32::consts::TAU;
        let radial = Vec2::new(angle.cos() * 470.0, angle.sin() * 238.0);
        let direction = radial.normalize_or_zero();
        let length = 72.0 + (index % 5) as f32 * 13.0;
        let thickness = if index % 4 == 0 { 4.0 } else { 2.0 };
        let ray = spawn_node(
            commands,
            table,
            Node {
                position_type: PositionType::Absolute,
                left: px(center.x + radial.x - length * 0.5),
                top: px(center.y + radial.y - thickness * 0.5),
                width: px(length),
                height: px(thickness),
                border_radius: BorderRadius::all(percent(50)),
                ..default()
            },
            Some(Color::NONE),
        );
        add_mahjong_win_stage_component(
            commands,
            ray,
            MahjongWinEffectTier::HighTotal,
            reveal_duration,
            start,
            duration,
            MahjongWinStageKind::FocusRay {
                delay: 0.12 + (index % 6) as f32 * 0.025,
                direction,
                phase,
            },
            100,
        );
    }
}

fn add_mahjong_major_stage_decorations(
    commands: &mut Commands,
    table: Entity,
    reveal_duration: f32,
    start: f32,
    duration: f32,
    glyph_count: usize,
) {
    let frame = spawn_node(
        commands,
        table,
        Node {
            position_type: PositionType::Absolute,
            left: px(18),
            right: px(18),
            top: px(16),
            bottom: px(16),
            border: UiRect::all(px(2)),
            border_radius: BorderRadius::all(px(14)),
            ..default()
        },
        None,
    );
    commands.entity(frame).insert(BorderColor::all(Color::NONE));
    add_mahjong_win_stage_component(
        commands,
        frame,
        MahjongWinEffectTier::MajorFan,
        reveal_duration,
        start + 0.72,
        duration - 0.72,
        MahjongWinStageKind::MajorFrame,
        108,
    );

    for (index, top) in [238.0, 442.0].into_iter().enumerate() {
        let sweep = spawn_node(
            commands,
            table,
            Node {
                position_type: PositionType::Absolute,
                left: px(180),
                top: px(top),
                width: px(DESIGN_WIDTH - 360.0),
                height: px(if index == 0 { 2.0 } else { 3.0 }),
                border_radius: BorderRadius::all(percent(50)),
                ..default()
            },
            Some(Color::NONE),
        );
        add_mahjong_win_stage_component(
            commands,
            sweep,
            MahjongWinEffectTier::MajorFan,
            reveal_duration,
            start,
            duration,
            MahjongWinStageKind::MajorSweep {
                delay: 0.74 + index as f32 * 0.08,
            },
            107,
        );
    }

    const SPARK_COUNT: usize = 24;
    let center = Vec2::new(DESIGN_WIDTH * 0.5, 340.0);
    for index in 0..SPARK_COUNT {
        let phase = index as f32 / SPARK_COUNT as f32;
        let angle = phase * std::f32::consts::TAU + 0.17;
        let radius = 128.0 + (index % 6) as f32 * 39.0;
        let offset = Vec2::new(angle.cos() * radius, angle.sin() * radius * 0.52);
        let size = 3.0 + (index % 3) as f32 * 1.5;
        let spark = spawn_node(
            commands,
            table,
            Node {
                position_type: PositionType::Absolute,
                left: px(center.x + offset.x - size * 0.5),
                top: px(center.y + offset.y - size * 0.5),
                width: px(size),
                height: px(size),
                border_radius: BorderRadius::all(px(1)),
                ..default()
            },
            Some(Color::NONE),
        );
        add_mahjong_win_stage_component(
            commands,
            spark,
            MahjongWinEffectTier::MajorFan,
            reveal_duration,
            start,
            duration,
            MahjongWinStageKind::MajorSpark {
                delay: 0.76 + (index % 8) as f32 * 0.07,
                drift: Vec2::new(angle.cos() * 38.0, angle.sin() * 24.0 - 14.0),
                phase,
            },
            106,
        );
    }

    for glyph_index in 0..glyph_count {
        let flash = spawn_node(
            commands,
            table,
            Node {
                position_type: PositionType::Absolute,
                left: px(0),
                right: px(0),
                top: px(0),
                bottom: px(0),
                ..default()
            },
            Some(Color::NONE),
        );
        add_mahjong_win_stage_component(
            commands,
            flash,
            MahjongWinEffectTier::MajorFan,
            reveal_duration,
            start,
            duration,
            MahjongWinStageKind::ImpactFlash {
                delay: MAJOR_FAN_GLYPH_DELAY + glyph_index as f32 * MAJOR_FAN_GLYPH_INTERVAL,
            },
            110,
        );
    }
}

#[allow(clippy::too_many_arguments)]
fn render_mahjong_win_stage(
    commands: &mut Commands,
    table: Entity,
    game: &MahjongSnapshot,
    own_seat: u8,
    result: &MahjongHandResultView,
    winner_index: usize,
    reveal_duration: f32,
    assets: &UiAssets,
    materials: &mut Assets<MahjongTileMaterial>,
) {
    let winner = &result.winners[winner_index];
    let tier = mahjong_win_effect_tier(winner);
    let start = mahjong_win_stage_start(result, winner_index);
    let duration = tier.presentation_duration();

    if tier == MahjongWinEffectTier::MajorFan {
        let backdrop = spawn_node(
            commands,
            table,
            Node {
                position_type: PositionType::Absolute,
                left: px(0),
                right: px(0),
                top: px(0),
                bottom: px(0),
                ..default()
            },
            Some(Color::NONE),
        );
        let backdrop_start = start + 0.72;
        add_mahjong_win_stage_component(
            commands,
            backdrop,
            tier,
            reveal_duration,
            backdrop_start,
            (start + duration - backdrop_start).max(0.01),
            MahjongWinStageKind::Backdrop,
            90,
        );
    }
    if tier == MahjongWinEffectTier::HighTotal {
        add_mahjong_high_focus_rays(commands, table, reveal_duration, start, duration);
    }

    let emphasized_tile_anchor = match tier {
        MahjongWinEffectTier::Normal | MahjongWinEffectTier::MajorFan => {
            mahjong_winning_tile_anchor(game, own_seat, winner)
        }
        MahjongWinEffectTier::HighTotal => None,
    };
    if let Some(anchor) = emphasized_tile_anchor {
        let tile_holder = spawn_node(
            commands,
            table,
            Node {
                position_type: PositionType::Absolute,
                left: px(anchor.x - 50.0),
                top: px(anchor.y - 70.0),
                width: px(100),
                height: px(140),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                overflow: Overflow::visible(),
                ..default()
            },
            None,
        );
        add_mahjong_tile_material(
            commands,
            tile_holder,
            MahjongTileVisual {
                kind: Some(winner.winning_tile.kind()),
                size: MahjongTileSize::OwnMeld,
                index: 0,
                highlighted: true,
                deal: None,
                relative: 0,
            },
            assets,
            materials,
        );
        add_mahjong_win_stage_component(
            commands,
            tile_holder,
            tier,
            reveal_duration,
            start,
            duration.min(if tier == MahjongWinEffectTier::MajorFan {
                0.84
            } else {
                duration
            }),
            MahjongWinStageKind::WinningTile,
            104,
        );
    }

    if matches!(
        tier,
        MahjongWinEffectTier::HighTotal | MahjongWinEffectTier::MajorFan
    ) {
        let hand_delay = if tier == MahjongWinEffectTier::MajorFan {
            0.78
        } else {
            0.10
        };
        render_mahjong_center_win_hand(
            commands,
            table,
            game,
            winner,
            tier,
            reveal_duration,
            start + hand_delay,
            duration - hand_delay,
            assets,
            materials,
        );
    }
    if tier == MahjongWinEffectTier::MajorFan {
        let Some(major_fan) = winner.score.fans.iter().max_by_key(|fan| fan.fan.points()) else {
            return;
        };
        let glyphs = major_fan.fan.name().chars().collect::<Vec<_>>();
        add_mahjong_major_stage_decorations(
            commands,
            table,
            reveal_duration,
            start,
            duration,
            glyphs.len(),
        );
        let glyph_width = 122.0;
        let total_width = glyphs.len() as f32 * glyph_width;
        let first_left = (DESIGN_WIDTH - total_width) * 0.5;
        for (glyph_index, glyph) in glyphs.into_iter().enumerate() {
            let holder = spawn_node(
                commands,
                table,
                Node {
                    position_type: PositionType::Absolute,
                    left: px(first_left + glyph_index as f32 * glyph_width),
                    top: px(265),
                    width: px(glyph_width),
                    height: px(136),
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    ..default()
                },
                None,
            );
            let text = add_text(
                commands,
                holder,
                glyph.to_string(),
                92.0,
                mahjong_win_effect_color(tier, false, 1.0),
                assets,
            );
            commands.entity(text).insert(TextShadow {
                offset: Vec2::new(4.0, 7.0),
                color: Color::BLACK.with_alpha(0.88),
            });
            commands
                .entity(holder)
                .insert((ZIndex(112), FocusPolicy::Pass));
            commands.entity(text).insert((
                MahjongWinFanGlyph {
                    reveal_duration,
                    start,
                    delay: MAJOR_FAN_GLYPH_DELAY + glyph_index as f32 * MAJOR_FAN_GLYPH_INTERVAL,
                },
                UiTransform::IDENTITY,
                Visibility::Hidden,
                FocusPolicy::Pass,
            ));
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn render_mahjong_center_win_hand(
    commands: &mut Commands,
    table: Entity,
    game: &MahjongSnapshot,
    winner: &MahjongWinView,
    tier: MahjongWinEffectTier,
    reveal_duration: f32,
    start: f32,
    duration: f32,
    assets: &UiAssets,
    materials: &mut Assets<MahjongTileMaterial>,
) {
    let Some(player) = game
        .players
        .iter()
        .find(|player| player.id == winner.player)
    else {
        return;
    };
    let Some(revealed) = &player.revealed_hand else {
        return;
    };
    let major = tier == MahjongWinEffectTier::MajorFan;
    let panel = spawn_node(
        commands,
        table,
        Node {
            position_type: PositionType::Absolute,
            left: px(0),
            right: px(0),
            top: px(if major { 252 } else { 248 }),
            height: px(if major { 176 } else { 184 }),
            padding: UiRect::axes(px(20), px(18)),
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            row_gap: px(12),
            overflow: Overflow::visible(),
            ..default()
        },
        Some(Color::NONE),
    );
    let content = spawn_node(
        commands,
        panel,
        Node {
            width: percent(100),
            height: percent(100),
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            row_gap: px(12),
            overflow: Overflow::visible(),
            ..default()
        },
        None,
    );
    add_text(
        commands,
        content,
        format!("{} 的和牌", player.name),
        18.0,
        mahjong_win_effect_color(tier, false, 1.0),
        assets,
    );
    let row = spawn_node(
        commands,
        content,
        Node {
            height: px(76),
            align_items: AlignItems::FlexEnd,
            justify_content: JustifyContent::Center,
            flex_direction: FlexDirection::Row,
            overflow: Overflow::visible(),
            ..default()
        },
        None,
    );
    let mut removed_winning_tile = false;
    for (index, tile) in revealed.iter().enumerate() {
        if !removed_winning_tile && *tile == winner.winning_tile {
            removed_winning_tile = true;
            continue;
        }
        add_mahjong_tile_material(
            commands,
            row,
            MahjongTileVisual {
                kind: Some(tile.kind()),
                size: MahjongTileSize::OwnMeld,
                index,
                highlighted: false,
                deal: None,
                relative: 0,
            },
            assets,
            materials,
        );
    }
    let gap = spawn_node(
        commands,
        row,
        Node {
            width: px(18),
            min_width: px(18),
            ..default()
        },
        None,
    );
    commands.entity(gap).insert(FocusPolicy::Pass);
    add_mahjong_tile_material(
        commands,
        row,
        MahjongTileVisual {
            kind: Some(winner.winning_tile.kind()),
            size: MahjongTileSize::OwnMeld,
            index: revealed.len(),
            highlighted: true,
            deal: None,
            relative: 0,
        },
        assets,
        materials,
    );
    if !major {
        add_mahjong_win_stage_component(
            commands,
            panel,
            tier,
            reveal_duration,
            start,
            duration,
            MahjongWinStageKind::Backdrop,
            101,
        );
    }
    add_mahjong_win_stage_component(
        commands,
        if major { content } else { row },
        tier,
        reveal_duration,
        start,
        duration,
        MahjongWinStageKind::Hand,
        102,
    );
}
