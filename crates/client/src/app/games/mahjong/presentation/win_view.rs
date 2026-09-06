use super::*;
use leocard_protocol::{MahjongHandResultView, MahjongWinView};
use leocard_protocol::{MahjongPhaseView, MahjongSnapshot};

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
        let Some(max_points) = winner.score.fans.iter().map(|fan| fan.fan.points()).max() else {
            continue;
        };
        let start = mahjong_win_stage_start(result, winner_index);
        for fan_index in 0..winner
            .score
            .fans
            .iter()
            .filter(|fan| fan.fan.points() == max_points)
            .count()
        {
            impacts.push(start + 1.02 + fan_index as f32 * 0.34);
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
        let Some(max_points) = winner.score.fans.iter().map(|fan| fan.fan.points()).max() else {
            return;
        };
        let major_fans = winner
            .score
            .fans
            .iter()
            .filter(|fan| fan.fan.points() == max_points)
            .collect::<Vec<_>>();
        let total_height = major_fans.len().saturating_sub(1) as f32 * 86.0;
        for (fan_index, fan) in major_fans.into_iter().enumerate() {
            let holder = spawn_node(
                commands,
                table,
                Node {
                    position_type: PositionType::Absolute,
                    left: px(290),
                    top: px(265.0 + fan_index as f32 * 86.0 - total_height * 0.5),
                    width: px(700),
                    height: px(110),
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    ..default()
                },
                None,
            );
            let text = add_text(
                commands,
                holder,
                format!("{}  {}番", fan.fan.name(), fan.points),
                64.0,
                mahjong_win_effect_color(tier, false, 1.0),
                assets,
            );
            commands.entity(text).insert(TextShadow {
                offset: Vec2::new(4.0, 7.0),
                color: Color::BLACK.with_alpha(0.88),
            });
            add_mahjong_win_stage_component(
                commands,
                holder,
                tier,
                reveal_duration,
                start,
                duration,
                MahjongWinStageKind::FanText {
                    delay: 1.02 + fan_index as f32 * 0.34,
                },
                112,
            );
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
        Some(if major {
            Color::NONE
        } else {
            Color::BLACK.with_alpha(0.86)
        }),
    );
    add_text(
        commands,
        panel,
        format!("{} 的和牌", player.name),
        18.0,
        mahjong_win_effect_color(tier, false, 1.0),
        assets,
    );
    let row = spawn_node(
        commands,
        panel,
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
    add_mahjong_win_stage_component(
        commands,
        panel,
        tier,
        reveal_duration,
        start,
        duration,
        MahjongWinStageKind::Hand,
        101,
    );
}
