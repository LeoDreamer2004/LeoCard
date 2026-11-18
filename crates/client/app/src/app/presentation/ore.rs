use bevy::image::{ImageLoaderSettings, ImageSampler};
use bevy::prelude::*;
use bevy::ui::VisualBox;
use leocard_ui::{ScrollbarSkin, TextInputSkin};

#[derive(Clone)]
pub(crate) struct OreUiSkin {
    frame: Handle<Image>,
    popup_frame: Handle<Image>,
    inset: Handle<Image>,
    cell: Handle<Image>,
    surface: Handle<Image>,
    scroll_track: Handle<Image>,
    scroll_thumb: Handle<Image>,
    disabled_scroll_thumb: Handle<Image>,
    button: Handle<Image>,
    disabled_button: Handle<Image>,
    highlighted_button: Handle<Image>,
    close_icon: Handle<Image>,
    input: Handle<Image>,
    focused_input: Handle<Image>,
}

impl OreUiSkin {
    pub(crate) fn load(asset_server: &AssetServer) -> Self {
        let load = |asset_path: &'static str| {
            asset_server
                .load_builder()
                .with_settings(|settings: &mut ImageLoaderSettings| {
                    settings.sampler = ImageSampler::nearest();
                })
                .load(asset_path)
        };
        Self {
            frame: load("ui/mc_guess/widgets/table-frame.png"),
            popup_frame: load("ui/mc_guess/widgets/popup-frame.png"),
            inset: load("ui/mc_guess/widgets/table-inset.png"),
            cell: load("ui/mc_guess/widgets/table-cell.png"),
            surface: load("ui/mc_guess/widgets/table-surface.png"),
            scroll_track: load("ui/mc_guess/widgets/scroll-track.png"),
            scroll_thumb: load("ui/mc_guess/widgets/scroll-thumb.png"),
            disabled_scroll_thumb: load("ui/mc_guess/widgets/scroll-thumb-disabled.png"),
            button: load("ui/mc_guess/widgets/button.png"),
            disabled_button: load("ui/mc_guess/widgets/button-disabled.png"),
            highlighted_button: load("ui/mc_guess/widgets/button-highlighted.png"),
            close_icon: load("ui/mc_guess/widgets/close.png"),
            input: load("ui/mc_guess/widgets/input.png"),
            focused_input: load("ui/mc_guess/widgets/input-focused.png"),
        }
    }

    pub(crate) fn close_icon(&self) -> Handle<Image> {
        self.close_icon.clone()
    }

    pub(crate) fn frame(&self) -> ImageNode {
        Self::image(self.frame.clone())
    }

    pub(crate) fn popup_frame(&self) -> ImageNode {
        Self::image(self.popup_frame.clone())
    }

    pub(crate) fn inset(&self) -> ImageNode {
        Self::recessed(self.inset.clone())
    }

    pub(crate) fn surface(&self) -> ImageNode {
        ImageNode::new(self.surface.clone())
            .with_mode(NodeImageMode::Stretch)
            .with_visual_box(VisualBox::PaddingBox)
    }

    pub(crate) fn cell(&self, tint: Color) -> ImageNode {
        ImageNode::new(self.cell.clone())
            .with_mode(NodeImageMode::Stretch)
            .with_color(tint)
            .with_visual_box(VisualBox::PaddingBox)
    }

    pub(crate) fn scrollbar(&self) -> ScrollbarSkin {
        ScrollbarSkin::textured(
            Self::recessed(self.scroll_track.clone()),
            Self::sliced(self.scroll_thumb.clone(), 1.0),
            Self::sliced(self.highlighted_button.clone(), 3.0),
            Self::sliced(self.disabled_scroll_thumb.clone(), 1.0),
        )
    }

    pub(crate) fn button(&self, enabled: bool, highlighted: bool) -> ImageNode {
        Self::sliced(
            if !enabled {
                self.disabled_button.clone()
            } else if highlighted {
                self.highlighted_button.clone()
            } else {
                self.button.clone()
            },
            3.0,
        )
    }

    pub(crate) fn input(&self) -> TextInputSkin {
        TextInputSkin::new(self.input_image(false), self.input_image(true))
    }

    pub(crate) fn input_image(&self, focused: bool) -> ImageNode {
        Self::sliced(
            if focused {
                self.focused_input.clone()
            } else {
                self.input.clone()
            },
            3.0,
        )
    }

    fn image(texture: Handle<Image>) -> ImageNode {
        Self::sliced(texture, 2.0)
    }

    fn recessed(texture: Handle<Image>) -> ImageNode {
        ImageNode::new(texture)
            .with_mode(NodeImageMode::Sliced(TextureSlicer {
                border: BorderRect {
                    min_inset: Vec2::new(1.0, 3.0),
                    max_inset: Vec2::ONE,
                },
                center_scale_mode: SliceScaleMode::Stretch,
                sides_scale_mode: SliceScaleMode::Stretch,
                max_corner_scale: 2.0,
            }))
            .with_visual_box(VisualBox::BorderBox)
    }

    fn sliced(texture: Handle<Image>, border: f32) -> ImageNode {
        ImageNode::new(texture)
            .with_mode(NodeImageMode::Sliced(TextureSlicer {
                border: BorderRect::all(border),
                center_scale_mode: SliceScaleMode::Stretch,
                sides_scale_mode: SliceScaleMode::Stretch,
                max_corner_scale: 2.0,
            }))
            .with_visual_box(VisualBox::BorderBox)
    }
}
