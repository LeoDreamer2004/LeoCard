use super::icons::EntryIcons;
use crate::app::presentation::{ScrollbarSkin, TextInputSkin};
use bevy::image::{ImageLoaderSettings, ImageSampler};
use bevy::prelude::*;
use bevy::ui::VisualBox;
use leocard_minecraft_guess::Entry;

#[derive(Resource)]
pub(crate) struct MinecraftGuessAssets {
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
    input: Handle<Image>,
    focused_input: Handle<Image>,
    icons: EntryIcons,
}

impl MinecraftGuessAssets {
    pub(super) fn icon(&self, entry: &Entry) -> ImageNode {
        self.icons.image(entry)
    }

    pub(super) fn frame(&self) -> ImageNode {
        Self::image(self.frame.clone())
    }

    pub(super) fn popup_frame(&self) -> ImageNode {
        Self::image(self.popup_frame.clone())
    }

    pub(super) fn inset(&self) -> ImageNode {
        Self::recessed(self.inset.clone())
    }

    pub(super) fn surface(&self) -> ImageNode {
        ImageNode::new(self.surface.clone())
            .with_mode(NodeImageMode::Stretch)
            .with_visual_box(VisualBox::PaddingBox)
    }

    pub(super) fn cell(&self, tint: Color) -> ImageNode {
        ImageNode::new(self.cell.clone())
            .with_mode(NodeImageMode::Stretch)
            .with_color(tint)
            .with_visual_box(VisualBox::PaddingBox)
    }

    pub(super) fn scrollbar(&self) -> ScrollbarSkin {
        ScrollbarSkin::textured(
            Self::recessed(self.scroll_track.clone()),
            Self::sliced(self.scroll_thumb.clone(), 1.0),
            Self::sliced(self.disabled_scroll_thumb.clone(), 1.0),
        )
    }

    pub(super) fn button(&self, enabled: bool, highlighted: bool) -> ImageNode {
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

    pub(super) fn input(&self) -> TextInputSkin {
        TextInputSkin::new(
            Self::sliced(self.input.clone(), 3.0),
            Self::sliced(self.focused_input.clone(), 3.0),
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

pub(super) fn load_guess_assets(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut layouts: ResMut<Assets<TextureAtlasLayout>>,
) {
    let load = |path: &'static str| {
        asset_server
            .load_builder()
            .with_settings(|settings: &mut ImageLoaderSettings| {
                settings.sampler = ImageSampler::nearest();
            })
            .load(path)
    };
    commands.insert_resource(MinecraftGuessAssets {
        frame: load("ui/minecraft_guess/table-frame.png"),
        popup_frame: load("ui/minecraft_guess/popup-frame.png"),
        inset: load("ui/minecraft_guess/table-inset.png"),
        cell: load("ui/minecraft_guess/table-cell.png"),
        surface: load("ui/minecraft_guess/table-surface.png"),
        scroll_track: load("ui/minecraft_guess/scroll-track.png"),
        scroll_thumb: load("ui/minecraft_guess/scroll-thumb.png"),
        disabled_scroll_thumb: load("ui/minecraft_guess/scroll-thumb-disabled.png"),
        button: load("ui/minecraft_guess/button.png"),
        disabled_button: load("ui/minecraft_guess/button-disabled.png"),
        highlighted_button: load("ui/minecraft_guess/button-highlighted.png"),
        input: load("ui/minecraft_guess/input.png"),
        focused_input: load("ui/minecraft_guess/input-focused.png"),
        icons: EntryIcons::new(load("ui/minecraft_guess/entries.png"), &mut layouts),
    });
}
