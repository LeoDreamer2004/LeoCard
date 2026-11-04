use super::scene::CategoryUnderline;
use crate::app::shell::{HomeCategory, UiState};
use bevy::prelude::*;

#[derive(Resource)]
pub(super) struct TabMotion {
    edges: Vec2,
    velocity: Vec2,
}

impl Default for TabMotion {
    fn default() -> Self {
        Self {
            edges: Vec2::new(14.0, 56.0),
            velocity: Vec2::ZERO,
        }
    }
}

pub(super) fn animate_category_underline(
    time: Res<Time>,
    ui: Res<UiState>,
    mut motion: ResMut<TabMotion>,
    mut lines: Query<&mut Node, With<CategoryUnderline>>,
) {
    if lines.is_empty() {
        return;
    }
    let slot = HomeCategory::ALL
        .iter()
        .position(|category| *category == ui.home.category)
        .unwrap_or(0);
    let target = Vec2::new(14.0, 56.0) + Vec2::splat(slot as f32 * 84.0);
    let direction = (target.x - motion.edges.x).signum();
    let stiffness = if direction >= 0.0 {
        Vec2::new(160.0, 300.0)
    } else {
        Vec2::new(300.0, 160.0)
    };
    let dt = time.delta_secs().min(0.05) / 4.0;
    for _ in 0..4 {
        let acceleration = (target - motion.edges) * stiffness - motion.velocity * 22.0;
        motion.velocity += acceleration * dt;
        let velocity = motion.velocity;
        motion.edges += velocity * dt;
    }
    for mut node in &mut lines {
        node.left = px(motion.edges.x.clamp(0.0, 280.0));
        node.width = px((motion.edges.y - motion.edges.x).clamp(20.0, 130.0));
    }
}
