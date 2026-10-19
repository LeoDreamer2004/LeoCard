use bevy::prelude::*;
use leocard_minecraft_guess::GraphView;
use std::f32::consts;

/// A local stress layout. Node IDs, arrows and guesses never depend on positions.
pub(super) fn initial_positions(graph: &GraphView) -> Vec<Vec2> {
    let n = graph.nodes.len();
    let mut distances = vec![vec![n as f32; n]; n];
    for (i, row) in distances.iter_mut().enumerate() {
        row[i] = 0.0;
    }
    for edge in &graph.edges {
        let a = usize::from(edge.source - 1);
        let b = usize::from(edge.target - 1);
        distances[a][b] = 1.0;
        distances[b][a] = 1.0;
    }
    for k in 0..n {
        for i in 0..n {
            for j in 0..n {
                distances[i][j] = distances[i][j].min(distances[i][k] + distances[k][j]);
            }
        }
    }
    let mut positions: Vec<_> = (0..n)
        .map(|i| {
            let angle = i as f32 * consts::TAU / n as f32;
            Vec2::new(angle.cos(), angle.sin()) * 200.0
        })
        .collect();
    for step in 0..360 {
        let mut forces = vec![Vec2::ZERO; n];
        for i in 0..n {
            for j in i + 1..n {
                let delta = positions[j] - positions[i];
                let length = delta.length().max(0.01);
                let target = distances[i][j] * 145.0;
                let force = delta / length
                    * ((length - target) / distances[i][j].powi(2) * 0.09
                        - (90.0 - length).max(0.0) * 0.3);
                forces[i] += force;
                forces[j] -= force;
            }
        }
        let speed = 1.0 - step as f32 / 450.0;
        for (p, force) in positions.iter_mut().zip(forces) {
            *p += force.clamp_length_max(18.0) * speed;
        }
    }
    let min = positions
        .iter()
        .copied()
        .reduce(Vec2::min)
        .expect("graph nodes");
    let max = positions
        .iter()
        .copied()
        .reduce(Vec2::max)
        .expect("graph nodes");
    positions
        .into_iter()
        .map(|p| Vec2::splat(0.09) + (p - min) / (max - min).max(Vec2::ONE) * 0.82)
        .collect()
}
