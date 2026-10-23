use bevy::prelude::*;

#[derive(Clone, Copy)]
pub(super) struct GraphLod {
    pub edge_opacity: f32,
    pub node_scale: f32,
    pub name_opacity: f32,
}

impl GraphLod {
    pub fn at_zoom(zoom: f32) -> Self {
        Self {
            edge_opacity: smooth_range(zoom, 0.78, 0.90),
            node_scale: 0.72 + 0.28 * smooth_range(zoom, 0.60, 0.74),
            name_opacity: smooth_range(zoom, 0.46, 0.56),
        }
    }

    pub fn name_transform(self) -> UiTransform {
        let inverse = self.node_scale.recip();
        UiTransform {
            // Counter-scale around the top center so the name stays below the icon.
            translation: Val2::new(percent(-50), percent((inverse - 1.0) * 50.0)),
            scale: Vec2::splat(inverse),
            ..default()
        }
    }
}

fn smooth_range(value: f32, hidden: f32, visible: f32) -> f32 {
    let amount = ((value - hidden) / (visible - hidden)).clamp(0.0, 1.0);
    amount * amount * (3.0 - 2.0 * amount)
}
