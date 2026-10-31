use bevy::prelude::*;

/// Apply logical wheel movement while keeping the viewport inside its content.
pub fn scroll_viewport_y(position: &mut ScrollPosition, viewport: &ComputedNode, delta: f32) {
    let maximum = ((viewport.content_size().y - viewport.size().y)
        * viewport.inverse_scale_factor())
    .max(0.0);
    position.y = (position.y - delta).clamp(0.0, maximum);
}
