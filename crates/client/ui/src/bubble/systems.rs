use super::{AnchoredBubble, BubblePlacement};
use crate::anchor_position_in_layer;
use crate::placement::clamp_overlay_origin;
use bevy::prelude::*;

fn bubble_origin(anchor: Vec2, layer_size: Vec2, size: Vec2) -> Vec2 {
    let x = if anchor.x < layer_size.x * 0.34 {
        anchor.x + 27.0
    } else if anchor.x > layer_size.x * 0.66 {
        anchor.x - size.x - 27.0
    } else {
        anchor.x - size.x * 0.5
    };
    clamp_bubble_origin(Vec2::new(x, anchor.y - 69.0), layer_size, size)
}

fn clamp_bubble_origin(origin: Vec2, layer_size: Vec2, size: Vec2) -> Vec2 {
    let margin = Vec2::splat(8.0).min(layer_size * 0.5);
    clamp_overlay_origin(
        origin,
        size,
        Rect::from_corners(margin, layer_size - margin),
    )
}

pub fn animate_bubbles(
    time: Res<Time>,
    mut commands: Commands,
    geometry: Query<(&ComputedNode, &UiGlobalTransform)>,
    mut bubbles: Query<(
        Entity,
        &ChildOf,
        &mut AnchoredBubble,
        &mut Node,
        &mut UiTransform,
        &mut ImageNode,
        &mut Visibility,
    )>,
    mut texts: Query<&mut TextColor>,
    mut images: Query<&mut ImageNode, Without<AnchoredBubble>>,
) {
    for (entity, parent, mut bubble, mut node, mut transform, mut background, mut visibility) in
        &mut bubbles
    {
        bubble.elapsed += time.delta_secs();
        if bubble.elapsed >= bubble.duration {
            commands.entity(entity).despawn();
            continue;
        }
        let position = geometry
            .get(parent.parent())
            .ok()
            .and_then(|(layer, layer_transform)| {
                let layer_size = layer.size() * layer.inverse_scale_factor();
                let size = geometry
                    .get(entity)
                    .ok()
                    .map(|(node, _)| node.size() * node.inverse_scale_factor())
                    .filter(|size| size.min_element() > 1.0)
                    .unwrap_or(bubble.size);
                match bubble.placement {
                    BubblePlacement::Anchor(anchor) | BubblePlacement::Beside(anchor) => {
                        let (anchor, anchor_transform) = geometry.get(anchor?).ok()?;
                        let position = anchor_position_in_layer(
                            layer,
                            layer_transform,
                            anchor,
                            anchor_transform,
                            bubble.anchor_offset,
                        )?;
                        Some(if matches!(bubble.placement, BubblePlacement::Beside(_)) {
                            clamp_bubble_origin(
                                position + Vec2::new(8.0, -size.y * 0.5),
                                layer_size,
                                size,
                            )
                        } else {
                            bubble_origin(position, layer_size, size)
                        })
                    }
                    BubblePlacement::TopLeft(inset) => {
                        Some(clamp_bubble_origin(inset, layer_size, size))
                    }
                    BubblePlacement::BottomRight(inset) => Some(clamp_bubble_origin(
                        layer_size - size - inset,
                        layer_size,
                        size,
                    )),
                }
            });
        if let Some(position) = position {
            bubble.missing_secs = 0.0;
            node.left = px(position.x);
            node.top = px(position.y);
            *visibility = Visibility::Inherited;
        } else {
            bubble.missing_secs += time.delta_secs();
            if bubble.missing_secs > 0.5 {
                commands.entity(entity).despawn();
                continue;
            }
        }
        let progress = (bubble.elapsed / 0.18).clamp(0.0, 1.0);
        let enter = 1.0 - (1.0 - progress).powi(3);
        let alpha = enter * ((bubble.duration - bubble.elapsed) / 0.48).clamp(0.0, 1.0);
        transform.translation = Val2::px(0.0, 9.0 * (1.0 - enter) - bubble.elapsed * 1.4);
        transform.scale = Vec2::splat(0.88 + enter * 0.12);
        background.color = bubble
            .background_color
            .with_alpha(bubble.background_color.alpha() * alpha);
        if let Some(text) = bubble.text.and_then(|id| texts.get_mut(id).ok()) {
            let mut color = text;
            color.0 = bubble
                .text_color
                .with_alpha(bubble.text_color.alpha() * alpha);
        }
        if let Some(mut image) = bubble.image.and_then(|id| images.get_mut(id).ok()) {
            image.color = Color::WHITE.with_alpha(alpha);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tall_bubble_fits_inside_the_overlay_near_a_lower_corner() {
        let size = Vec2::new(86.0, 86.0);
        let origin = bubble_origin(Vec2::new(398.0, 298.0), Vec2::new(400.0, 300.0), size);
        assert!(origin.x >= 8.0 && origin.x + size.x <= 392.0);
        assert!(origin.y >= 8.0 && origin.y + size.y <= 292.0);
    }
}
