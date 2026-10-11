use bevy::prelude::*;

#[derive(Component)]
pub(crate) struct Scrollbar {
    pub(super) scroll: Entity,
    pub(super) thumb: Entity,
    pub(super) skin: ScrollbarSkin,
}

#[derive(Resource, Default)]
pub(crate) struct ScrollbarDrag {
    pub(super) active: Option<(Entity, f32)>,
}

pub(crate) enum ScrollbarSkin {
    Colored(Color),
    Textured(Box<TexturedScrollbar>),
}

pub(crate) struct TexturedScrollbar {
    pub(super) track: ImageNode,
    pub(super) thumb: ImageNode,
    pub(super) disabled_thumb: ImageNode,
}

impl ScrollbarSkin {
    pub(crate) fn textured(track: ImageNode, thumb: ImageNode, disabled_thumb: ImageNode) -> Self {
        Self::Textured(Box::new(TexturedScrollbar {
            track,
            thumb,
            disabled_thumb,
        }))
    }
}
