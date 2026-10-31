//! Geometry shared by anchored overlays, in the caller's coordinate space.

use bevy::prelude::*;

pub(crate) fn clamp_overlay_origin(origin: Vec2, size: Vec2, bounds: Rect) -> Vec2 {
    origin.clamp(bounds.min, (bounds.max - size).max(bounds.min))
}

/// Project an anchor's center into the logical coordinates of an overlay layer.
pub fn anchor_position_in_layer(
    layer: &ComputedNode,
    layer_transform: &UiGlobalTransform,
    anchor: &ComputedNode,
    anchor_transform: &UiGlobalTransform,
    relative_offset: Vec2,
) -> Option<Vec2> {
    if layer.size().min_element() <= 1.0 || anchor.size().min_element() <= 1.0 {
        return None;
    }
    let inverse = layer_transform.try_inverse()?;
    let center = anchor_transform.to_scale_angle_translation().2;
    let position =
        inverse.transform_point2(center) + layer.size() * 0.5 + anchor.size() * relative_offset;
    Some(position * layer.inverse_scale_factor())
}
