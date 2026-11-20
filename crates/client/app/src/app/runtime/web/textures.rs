use bevy::prelude::*;
use bevy::render::render_resource::TextureFormat;

/// WebGL lacks normalized 16-bit textures. Preserve their linear interpretation at 8 bits.
pub(super) fn normalize_browser_images(mut images: ResMut<Assets<Image>>) {
    let unsupported = images
        .iter()
        .filter_map(|(id, image)| {
            compatible_format(image.texture_descriptor.format).map(|format| (id, format))
        })
        .collect::<Vec<_>>();
    for (id, format) in unsupported {
        if let Some(mut image) = images.get_mut(id) {
            if let Some(bytes) = &image.data {
                let pixels = bytes
                    .as_chunks::<2>()
                    .0
                    .iter()
                    .map(|bytes| {
                        let value = u32::from(u16::from_le_bytes([bytes[0], bytes[1]]));
                        ((value * 255 + 32767) / 65535) as u8
                    })
                    .collect();
                image.data = Some(pixels);
            }
            image.texture_descriptor.format = format;
        }
    }
}

fn compatible_format(format: TextureFormat) -> Option<TextureFormat> {
    Some(match format {
        TextureFormat::R16Unorm => TextureFormat::R8Unorm,
        TextureFormat::Rg16Unorm => TextureFormat::Rg8Unorm,
        TextureFormat::Rgba16Unorm => TextureFormat::Rgba8Unorm,
        _ => return None,
    })
}
