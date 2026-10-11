use crate::app::runtime::UiAssets;
use bevy::{prelude::*, ui::VisualBox};

#[derive(Component, Clone)]
pub(crate) struct TextInputSkin {
    idle: ImageNode,
    focused: ImageNode,
}

impl TextInputSkin {
    pub(crate) fn new(idle: ImageNode, focused: ImageNode) -> Self {
        Self { idle, focused }
    }

    pub(super) fn cozy(assets: &UiAssets) -> Self {
        let image = |texture| {
            ImageNode::new(texture)
                .with_mode(NodeImageMode::Sliced(TextureSlicer {
                    border: BorderRect::all(32.0),
                    center_scale_mode: SliceScaleMode::Stretch,
                    sides_scale_mode: SliceScaleMode::Stretch,
                    max_corner_scale: 0.45,
                }))
                .with_visual_box(VisualBox::BorderBox)
        };
        Self::new(
            image(assets.home.input.clone()),
            image(assets.home.focused_input.clone()),
        )
    }

    pub(super) fn image(&self, focused: bool) -> &ImageNode {
        if focused { &self.focused } else { &self.idle }
    }
}
