use super::super::shape::NodeShape;
use super::curve::CurveGeometry;
use bevy::prelude::*;
use leocard_mc_guess::McGuessGraphView;
use std::collections::HashMap;

/// A fixed lane per relation keeps curves continuous while either endpoint moves.
pub(in super::super) fn route_curves(
    graph: &McGuessGraphView,
    positions: &[Vec2],
    node_scale: f32,
) -> Vec<CurveGeometry> {
    let mut lanes = HashMap::new();
    graph
        .edges
        .iter()
        .map(|edge| {
            let a = usize::from(edge.source - 1);
            let b = usize::from(edge.target - 1);
            let lane = lanes.entry((a.min(b), a.max(b))).or_insert(0);
            let offset = (18.0 + *lane as f32 * 24.0)
                * if *lane % 2 == 0 { 1.0 } else { -1.0 }
                * if a < b { 1.0 } else { -1.0 };
            *lane += 1;
            CurveGeometry::between(
                positions[a],
                positions[b],
                offset,
                NodeShape::for_kind(graph.nodes[a].kind),
                NodeShape::for_kind(graph.nodes[b].kind),
                node_scale,
            )
        })
        .collect()
}
