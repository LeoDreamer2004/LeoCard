use super::super::geometry::Segment;
use super::solver::{EDGE_CLEARANCE, EDGE_LENGTH, MeshLayout, NODE_SPACING};
use bevy::prelude::*;

impl MeshLayout {
    pub fn quality(&self, positions: &[Vec2]) -> f32 {
        let mut score = 0.0;
        for i in 0..positions.len() {
            for j in i + 1..positions.len() {
                let delta = positions[j] - positions[i];
                let hops = self.distances[i][j];
                score +=
                    (((delta / self.shape).length() - hops * EDGE_LENGTH) / hops).powi(2) * 0.02;
                let overlap = NODE_SPACING - delta.abs();
                if overlap.min_element() > 0.0 {
                    score += overlap.min_element().powi(2) * 20.0;
                }
            }
        }
        for (index, &(a, b)) in self.edges.iter().enumerate() {
            let segment = Segment::new(positions[a], positions[b]);
            for (i, &position) in positions.iter().enumerate() {
                if i != a && i != b {
                    let (closest, _) = segment.closest(position);
                    score += (EDGE_CLEARANCE - position.distance(closest))
                        .max(0.0)
                        .powi(2)
                        * 4.0;
                }
            }
            for &(c, d) in &self.edges[index + 1..] {
                if a != c
                    && a != d
                    && b != c
                    && b != d
                    && segment.crosses(Segment::new(positions[c], positions[d]))
                {
                    score += 100_000.0;
                }
            }
        }
        score
    }
}
