use bevy::prelude::*;
use bevy::ui::VisualBox;

pub(crate) fn sliced_image(texture: Handle<Image>, border: f32, corner_scale: f32) -> ImageNode {
    let mut image = ImageNode::new(texture).with_mode(NodeImageMode::Sliced(TextureSlicer {
        border: BorderRect::all(border),
        center_scale_mode: SliceScaleMode::Stretch,
        sides_scale_mode: SliceScaleMode::Stretch,
        max_corner_scale: corner_scale,
    }));
    image.visual_box = VisualBox::BorderBox;
    image
}
