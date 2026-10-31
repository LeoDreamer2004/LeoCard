use bevy::prelude::*;
use fastrand::Rng;
use leocard_mc_guess::McGuessGraphView;
use std::f32::consts;

pub(super) const NODE_SPACING: Vec2 = Vec2::new(104.0, 96.0);
pub(super) const EDGE_LENGTH: f32 = 135.0;
pub(super) const EDGE_CLEARANCE: f32 = 64.0;

pub(super) struct MeshLayout {
    pub distances: Vec<Vec<f32>>,
    pub edges: Vec<(usize, usize)>,
    pub shape: Vec2,
}

/// Positions stay in logical pixels: adding vertices never squeezes existing gaps.
pub(in super::super) fn initial_positions(graph: &McGuessGraphView, viewport: Vec2) -> Vec<Vec2> {
    let layout = MeshLayout::new(graph, viewport);
    let n = graph.nodes.len();
    if n == 0 {
        return Vec::new();
    }
    let mut best = Vec::new();
    let mut best_score = f32::INFINITY;
    for attempt in 0..3 {
        let mut positions: Vec<_> = (0..n)
            .map(|i| {
                let angle = i as f32 * consts::TAU / n as f32;
                Vec2::new(angle.cos(), angle.sin()) * layout.shape * (n as f32).sqrt() * 62.0
            })
            .collect();
        if attempt != 0 {
            Rng::with_seed(17 + attempt).shuffle(&mut positions);
        }
        layout.relax(&mut positions, 360, false);
        layout.uncross(&mut positions);
        let mut refined = positions.clone();
        layout.relax(&mut refined, 120, true);
        layout.uncross(&mut refined);
        layout.orient(&mut positions);
        layout.orient(&mut refined);
        layout.spread(&mut positions);
        layout.spread(&mut refined);
        if layout.quality(&refined) < layout.quality(&positions) {
            positions = refined;
        }
        let score = layout.quality(&positions);
        if score < best_score {
            best_score = score;
            best = positions;
        }
    }
    // The graph can extend beyond the viewport; keep the initial question visible.
    let center = best[usize::from(graph.selected - 1)];
    for position in &mut best {
        *position -= center;
    }
    best
}

impl MeshLayout {
    fn new(graph: &McGuessGraphView, viewport: Vec2) -> Self {
        let n = graph.nodes.len();
        let mut distances = vec![vec![n as f32; n]; n];
        let mut edges = Vec::new();
        for (i, row) in distances.iter_mut().enumerate() {
            row[i] = 0.0;
        }
        for edge in &graph.edges {
            let a = usize::from(edge.source - 1);
            let b = usize::from(edge.target - 1);
            distances[a][b] = 1.0;
            distances[b][a] = 1.0;
            let pair = (a.min(b), a.max(b));
            if a != b && !edges.contains(&pair) {
                edges.push(pair);
            }
        }
        for k in 0..n {
            for i in 0..n {
                for j in 0..n {
                    distances[i][j] = distances[i][j].min(distances[i][k] + distances[k][j]);
                }
            }
        }
        // Keep area and legible spacing, while encouraging a taller arrangement
        // in narrow panes. This shapes the solve instead of squeezing its result.
        let aspect = (viewport.x / viewport.y).clamp(0.5, 1.6).sqrt();
        Self {
            distances,
            edges,
            shape: Vec2::new(aspect, aspect.recip()),
        }
    }

    fn spread(&self, positions: &mut [Vec2]) {
        let mut scale = 1.0_f32;
        for i in 0..positions.len() {
            for j in i + 1..positions.len() {
                let delta = (positions[j] - positions[i]).abs().max(Vec2::splat(0.01));
                scale = scale.max((NODE_SPACING / delta).min_element());
            }
        }
        for position in positions {
            *position *= scale;
        }
    }

    fn uncross(&self, positions: &mut [Vec2]) {
        let mut score = self.quality(positions);
        // Swaps can escape a crossed local minimum without a slow global search.
        for i in 0..positions.len() {
            for j in i + 1..positions.len() {
                positions.swap(i, j);
                let candidate = self.quality(positions);
                if candidate < score {
                    score = candidate;
                } else {
                    positions.swap(i, j);
                }
            }
        }
    }
}
