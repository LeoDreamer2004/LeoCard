use super::super::geometry::Segment;
use super::solver::{EDGE_CLEARANCE, EDGE_LENGTH, MeshLayout, NODE_SPACING};
use bevy::prelude::*;

impl MeshLayout {
    pub fn relax(&self, positions: &mut [Vec2], steps: usize, avoid_obstacles: bool) {
        let n = positions.len();
        let mut forces = vec![Vec2::ZERO; n];
        for step in 0..steps {
            forces.fill(Vec2::ZERO);
            for i in 0..n {
                for j in i + 1..n {
                    let delta = positions[j] - positions[i];
                    let shaped = delta / self.shape;
                    let length = shaped.length().max(0.01);
                    let hops = self.distances[i][j];
                    let mut force = shaped / length / self.shape * (length - hops * EDGE_LENGTH)
                        / hops.powi(2)
                        * 0.06;
                    let overlap = NODE_SPACING - delta.abs();
                    if avoid_obstacles && overlap.min_element() > 0.0 {
                        let axis = if overlap.x < overlap.y {
                            Vec2::X
                        } else {
                            Vec2::Y
                        };
                        let sign = if delta.dot(axis) < 0.0 { -1.0 } else { 1.0 };
                        force -= axis * sign * overlap.min_element() * 0.65;
                    }
                    forces[i] += force;
                    forces[j] -= force;
                }
            }
            for &(a, b) in self.edges.iter().filter(|_| avoid_obstacles) {
                let segment = Segment::new(positions[a], positions[b]);
                for i in 0..n {
                    if i == a || i == b {
                        continue;
                    }
                    let (closest, t) = segment.closest(positions[i]);
                    if !(0.05..0.95).contains(&t) {
                        continue;
                    }
                    let delta = positions[i] - closest;
                    let distance = delta.length();
                    if distance < EDGE_CLEARANCE {
                        let direction = if distance > 0.01 {
                            delta / distance
                        } else {
                            (positions[b] - positions[a]).normalize_or_zero().perp()
                        };
                        let force = direction * (EDGE_CLEARANCE - distance) * 0.35;
                        forces[i] += force;
                        forces[a] -= force * (1.0 - t) * 0.5;
                        forces[b] -= force * t * 0.5;
                    }
                }
            }
            let speed = 1.0 - step as f32 / steps as f32 * 0.8;
            for (position, force) in positions.iter_mut().zip(&forces) {
                *position += force.clamp_length_max(14.0) * speed;
            }
        }
    }
}
