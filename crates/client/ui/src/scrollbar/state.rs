use bevy::prelude::*;

/// Explicit logical content height for viewports whose layout overflow includes extra space.
#[derive(Component, Default)]
pub struct ScrollContentHeight(pub f32);

impl ScrollContentHeight {
    pub fn maximum(&self, viewport: &ComputedNode) -> f32 {
        (self.0 - viewport.size().y * viewport.inverse_scale_factor()).max(0.0)
    }
}

#[derive(Component)]
pub struct Scrollbar {
    pub(super) scroll: Entity,
    pub(super) thumb: Entity,
    pub(super) skin: ScrollbarSkin,
}

#[derive(Resource, Default)]
pub struct ScrollbarDrag {
    pub(super) active: Option<(Entity, f32)>,
}

pub enum ScrollbarSkin {
    Colored(Color),
    Textured(Box<TexturedScrollbar>),
}

pub struct TexturedScrollbar {
    pub(super) track: ImageNode,
    pub(super) thumb: ImageNode,
    pub(super) highlighted_thumb: ImageNode,
    pub(super) disabled_thumb: ImageNode,
}

impl ScrollbarSkin {
    pub fn textured(
        track: ImageNode,
        thumb: ImageNode,
        highlighted_thumb: ImageNode,
        disabled_thumb: ImageNode,
    ) -> Self {
        Self::Textured(Box::new(TexturedScrollbar {
            track,
            thumb,
            highlighted_thumb,
            disabled_thumb,
        }))
    }
}
