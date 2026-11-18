use super::assets::DrawGuessAssets;
use crate::app::presentation::GAME_SIDE_CONTROL_SCALE;
use bevy::prelude::*;
use bevy::render::render_resource::Extent3d;
use leocard_ui::{TextInputSkin, sliced_image};

pub(super) struct DrawingControlSkin {
    normal: Handle<Image>,
    disabled: Handle<Image>,
    highlighted: Handle<Image>,
    input: Handle<Image>,
    focused_input: Handle<Image>,
}

impl DrawingControlSkin {
    pub fn new(images: &Assets<Image>) -> Self {
        Self {
            normal: images.reserve_handle(),
            disabled: images.reserve_handle(),
            highlighted: images.reserve_handle(),
            input: images.reserve_handle(),
            focused_input: images.reserve_handle(),
        }
    }

    pub fn button(&self, enabled: bool, highlighted: bool) -> ImageNode {
        let texture = if !enabled {
            &self.disabled
        } else if highlighted {
            &self.highlighted
        } else {
            &self.normal
        };
        Self::image(texture)
    }

    pub fn input(&self) -> TextInputSkin {
        TextInputSkin::new(Self::image(&self.input), Self::image(&self.focused_input))
    }

    fn image(texture: &Handle<Image>) -> ImageNode {
        sliced_image(texture.clone(), 3.0, GAME_SIDE_CONTROL_SCALE)
    }

    fn compact(source: &Image) -> Image {
        let side = source.height();
        let mut compact = source.clone();
        compact.resize(Extent3d {
            width: side,
            ..source.texture_descriptor.size
        });
        // Keep both ends verbatim and omit only the long, uniform middle strip.
        // A square source keeps nine-sliced borders independent of control width.
        for y in 0..side {
            for x in 0..side {
                let source_x = if x < side / 2 {
                    x
                } else {
                    source.width() - side + x
                };
                let pixel = source
                    .pixel_bytes(UVec3::new(source_x, y, 0))
                    .expect("OreUI control source has readable pixels");
                compact
                    .pixel_bytes_mut(UVec3::new(x, y, 0))
                    .expect("compact control pixel is within the square")
                    .copy_from_slice(pixel);
            }
        }
        compact
    }
}

pub(super) fn prepare(art: Res<DrawGuessAssets>, mut images: ResMut<Assets<Image>>) {
    if images.contains(&art.controls.normal) {
        return;
    }
    let sources = [
        art.skin.button(true, false).image,
        art.skin.button(false, false).image,
        art.skin.button(true, true).image,
        art.skin.input_image(false).image,
        art.skin.input_image(true).image,
    ];
    let [
        Some(normal),
        Some(disabled),
        Some(highlighted),
        Some(input),
        Some(focused_input),
    ] = sources.each_ref().map(|handle| images.get(handle))
    else {
        return;
    };
    let compact =
        [normal, disabled, highlighted, input, focused_input].map(DrawingControlSkin::compact);
    for (handle, image) in [
        &art.controls.normal,
        &art.controls.disabled,
        &art.controls.highlighted,
        &art.controls.input,
        &art.controls.focused_input,
    ]
    .into_iter()
    .zip(compact)
    {
        images
            .insert(handle.id(), image)
            .expect("compact control handles were reserved from these assets");
    }
}
