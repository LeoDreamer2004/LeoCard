use crate::placement::clamp_overlay_origin;
use bevy::prelude::*;
use bevy::ui::ComputedUiRenderTargetInfo;

const WINDOW_MARGIN: f32 = 8.0;
const ANCHOR_GAP: f32 = 6.0;

/// The popup owns placement; callers only identify the element it belongs to.
#[derive(Component)]
pub struct AnchoredPopup(pub Entity);

struct PopupGeometry {
    anchor: Rect,
    size: Vec2,
    bounds: Rect,
    gap: f32,
}

impl PopupGeometry {
    fn origin(&self) -> Vec2 {
        let below = self.anchor.max.y + self.gap;
        let above = self.anchor.min.y - self.gap - self.size.y;
        let bottom_space = self.bounds.max.y - below;
        let top_space = self.anchor.min.y - self.gap - self.bounds.min.y;
        let top = if bottom_space >= self.size.y || bottom_space >= top_space {
            below
        } else {
            above
        };
        let desired = Vec2::new(self.anchor.center().x - self.size.x * 0.5, top);
        clamp_overlay_origin(desired, self.size, self.bounds)
    }
}

/// Layout computes fresh transforms each frame. Apply one screen-space translation
/// to the whole popup before clipping and rendering, avoiding a frame of overflow
/// or a feedback loop from changing layout offsets after measuring them.
pub fn position_anchored_popups(
    popups: Query<(Entity, &AnchoredPopup, &ComputedUiRenderTargetInfo)>,
    nodes: Query<&ComputedNode>,
    mut transforms: Query<&mut UiGlobalTransform>,
    children: Query<&Children>,
) {
    for (entity, popup, target) in &popups {
        let (Ok(node), Ok(anchor_node)) = (nodes.get(entity), nodes.get(popup.0)) else {
            continue;
        };
        if node.size().min_element() <= 0.0 || anchor_node.size().min_element() <= 0.0 {
            continue;
        }
        let (Ok(transform), Ok(anchor_transform)) =
            (transforms.get(entity), transforms.get(popup.0))
        else {
            continue;
        };
        let rect = node_rect(node, transform);
        let viewport = target.physical_size().as_vec2();
        let margin = Vec2::splat(WINDOW_MARGIN * target.scale_factor()).min(viewport * 0.5);
        let geometry = PopupGeometry {
            anchor: node_rect(anchor_node, anchor_transform),
            size: rect.size(),
            bounds: Rect::from_corners(margin, viewport - margin),
            gap: ANCHOR_GAP * target.scale_factor(),
        };
        let offset = geometry.origin() - rect.min;
        if offset.length_squared() < 0.0001 {
            continue;
        }
        let translation = UiGlobalTransform::from_translation(offset);
        for child in [entity]
            .into_iter()
            .chain(children.iter_descendants(entity))
        {
            if let Ok(mut transform) = transforms.get_mut(child) {
                *transform = translation * *transform;
            }
        }
    }
}

fn node_rect(node: &ComputedNode, transform: &UiGlobalTransform) -> Rect {
    let transform = transform.affine();
    let half_size = transform.matrix2.abs() * node.size() * 0.5;
    Rect::from_corners(
        transform.translation - half_size,
        transform.translation + half_size,
    )
}
