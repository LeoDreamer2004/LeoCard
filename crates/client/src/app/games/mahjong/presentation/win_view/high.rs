use super::super::{MahjongWinEffectTier, MahjongWinStageKind};
use super::{WinStagePartSpec, add_win_stage_component};
use crate::app::presentation::{DESIGN_WIDTH, spawn_node};
use bevy::prelude::*;

pub(super) fn add_high_focus_rays(
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
        add_win_stage_component(
            commands,
            ray,
            WinStagePartSpec {
                tier: MahjongWinEffectTier::HighTotal,
                reveal_duration,
                start,
                duration,
                kind: MahjongWinStageKind::FocusRay {
                    delay: 0.12 + (index % 6) as f32 * 0.025,
                    direction,
                    phase,
                },
                z_index: 100,
            },
        );
    }
}
