use super::super::{MahjongWinEffectTier, MahjongWinFanGlyph, MahjongWinStageKind};
use super::{
    MAJOR_FAN_GLYPH_DELAY, MAJOR_FAN_GLYPH_INTERVAL, WinStagePartSpec, add_win_stage_component,
    mahjong_win_effect_color,
};
use crate::app::presentation::{DESIGN_WIDTH, add_text, spawn_node};
use crate::app::runtime::UiAssets;
use bevy::prelude::*;
use bevy::ui::FocusPolicy;
use leocard_protocol::MahjongWinView;

pub(super) fn add_major_stage_decorations(
    commands: &mut Commands,
    table: Entity,
    winner: &MahjongWinView,
    reveal_duration: f32,
    start: f32,
    duration: f32,
    assets: &UiAssets,
) {
    let Some(major_fan) = winner.score.fans.iter().max_by_key(|fan| fan.fan.points()) else {
        return;
    };
    let glyphs = major_fan.fan.name().chars().collect::<Vec<_>>();
    add_frame(commands, table, reveal_duration, start, duration);
    add_sweeps(commands, table, reveal_duration, start, duration);
    add_sparks(commands, table, reveal_duration, start, duration);
    add_impact_flashes(
        commands,
        table,
        reveal_duration,
        start,
        duration,
        glyphs.len(),
    );
    add_glyphs(commands, table, reveal_duration, start, &glyphs, assets);
}

fn add_frame(
    commands: &mut Commands,
    table: Entity,
    reveal_duration: f32,
    start: f32,
    duration: f32,
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
    add_win_stage_component(
        commands,
        frame,
        WinStagePartSpec {
            tier: MahjongWinEffectTier::MajorFan,
            reveal_duration,
            start: start + 0.72,
            duration: duration - 0.72,
            kind: MahjongWinStageKind::MajorFrame,
            z_index: 108,
        },
    );
}

fn add_sweeps(
    commands: &mut Commands,
    table: Entity,
    reveal_duration: f32,
    start: f32,
    duration: f32,
) {
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
        add_win_stage_component(
            commands,
            sweep,
            WinStagePartSpec {
                tier: MahjongWinEffectTier::MajorFan,
                reveal_duration,
                start,
                duration,
                kind: MahjongWinStageKind::MajorSweep {
                    delay: 0.74 + index as f32 * 0.08,
                },
                z_index: 107,
            },
        );
    }
}

fn add_sparks(
    commands: &mut Commands,
    table: Entity,
    reveal_duration: f32,
    start: f32,
    duration: f32,
) {
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
        add_win_stage_component(
            commands,
            spark,
            WinStagePartSpec {
                tier: MahjongWinEffectTier::MajorFan,
                reveal_duration,
                start,
                duration,
                kind: MahjongWinStageKind::MajorSpark {
                    delay: 0.76 + (index % 8) as f32 * 0.07,
                    drift: Vec2::new(angle.cos() * 38.0, angle.sin() * 24.0 - 14.0),
                    phase,
                },
                z_index: 106,
            },
        );
    }
}

fn add_impact_flashes(
    commands: &mut Commands,
    table: Entity,
    reveal_duration: f32,
    start: f32,
    duration: f32,
    glyph_count: usize,
) {
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
        add_win_stage_component(
            commands,
            flash,
            WinStagePartSpec {
                tier: MahjongWinEffectTier::MajorFan,
                reveal_duration,
                start,
                duration,
                kind: MahjongWinStageKind::ImpactFlash {
                    delay: MAJOR_FAN_GLYPH_DELAY + glyph_index as f32 * MAJOR_FAN_GLYPH_INTERVAL,
                },
                z_index: 110,
            },
        );
    }
}

fn add_glyphs(
    commands: &mut Commands,
    table: Entity,
    reveal_duration: f32,
    start: f32,
    glyphs: &[char],
    assets: &UiAssets,
) {
    let glyph_width = 122.0;
    let first_left = (DESIGN_WIDTH - glyphs.len() as f32 * glyph_width) * 0.5;
    for (glyph_index, glyph) in glyphs.iter().copied().enumerate() {
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
            mahjong_win_effect_color(MahjongWinEffectTier::MajorFan, false, 1.0),
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
