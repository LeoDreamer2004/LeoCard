use super::solver::{MeshLayout, NODE_SPACING};
use bevy::prelude::*;
use std::f32::consts;

impl MeshLayout {
    pub fn orient(&self, positions: &mut [Vec2]) {
        let mut best = Rot2::IDENTITY;
        let mut cost = f32::INFINITY;
        // Rotation preserves the network and its crossings. Only its occupied
        // envelope is compared; final spacing is handled by the existing spread.
        for step in 0..12 {
            let rotation = Rot2::radians(step as f32 * consts::PI / 12.0);
            let mut min = Vec2::splat(f32::INFINITY);
            let mut max = Vec2::splat(f32::NEG_INFINITY);
            for &point in positions.iter() {
                let rotated = rotation * point;
                min = min.min(rotated);
                max = max.max(rotated);
            }
            let extent = (max - min + NODE_SPACING) / self.shape;
            let candidate = extent.max_element();
            if candidate < cost {
                cost = candidate;
                best = rotation;
            }
        }
        for point in positions {
            *point = best * *point;
        }
    }
}
