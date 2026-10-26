use crate::{UiComposer, ui_component};
use bevy::picking::Pickable;
use bevy::prelude::*;
use bevy::ui::VisualBox;

pub fn sliced_image(texture: Handle<Image>, border: f32, corner_scale: f32) -> ImageNode {
    let mut image = ImageNode::new(texture).with_mode(NodeImageMode::Sliced(TextureSlicer {
        border: BorderRect::all(border),
        center_scale_mode: SliceScaleMode::Stretch,
        sides_scale_mode: SliceScaleMode::Stretch,
        max_corner_scale: corner_scale,
    }));
    image.visual_box = VisualBox::BorderBox;
    image
}

/// Attach a non-interactive skin without changing the panel's content layout.
pub fn decorate_panel(commands: &mut UiComposer, panel: Entity, image: ImageNode) -> Entity {
    commands
        .entity(panel)
        .insert((BackgroundColor(Color::NONE), BorderColor::all(Color::NONE)));
    commands.mount(
        panel,
        "skin",
        bsn! {
            Node {
                position_type: PositionType::Absolute,
                left: px(0), right: px(0), top: px(0), bottom: px(0),
            }
            @ui_component(image)
            ZIndex(-1)
            Pickable::IGNORE
        },
    )
}
