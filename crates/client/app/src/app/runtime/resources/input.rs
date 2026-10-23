use super::UiAssets;
use crate::app::presentation::{MUTED, TEXT};
use bevy::prelude::*;
use bevy::ui::VisualBox;
use leocard_ui::{TextInputSkin, TextInputStyle};

impl UiAssets {
    pub(crate) fn text_input_style(&self) -> TextInputStyle {
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
        TextInputStyle {
            font: self.font.clone(),
            text: TEXT,
            placeholder: MUTED,
            skin: TextInputSkin::new(
                image(self.home.input.clone()),
                image(self.home.focused_input.clone()),
            ),
        }
    }
}
