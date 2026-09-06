//! 用户图片规范化与运行时生成的遮罩纹理。

use super::*;
use bevy::asset::RenderAssetUsages;
use leocard_protocol::{AVATAR_DIMENSION, MAX_AVATAR_BYTES};
use std::fs;
use std::io::Cursor;
use std::path::Path;

pub fn normalize_avatar(path: &Path) -> Result<Vec<u8>, String> {
    let source = fs::read(path).map_err(|error| format!("无法读取头像文件：{error}"))?;
    normalize_avatar_bytes(&source)
}

pub fn normalize_avatar_bytes(source: &[u8]) -> Result<Vec<u8>, String> {
    // Input may be PNG or JPEG, but local persistence and the wire format remain
    // one bounded square PNG representation.
    let image = image::load_from_memory(source)
        .map_err(|_| "头像必须是有效的 PNG、JPG 或 JPEG 图片".to_owned())?;
    let image = image.resize_to_fill(
        AVATAR_DIMENSION,
        AVATAR_DIMENSION,
        image::imageops::FilterType::Lanczos3,
    );
    let mut output = Cursor::new(Vec::new());
    image
        .write_to(&mut output, image::ImageFormat::Png)
        .map_err(|error| format!("头像压缩失败：{error}"))?;
    let png = output.into_inner();
    if png.len() > MAX_AVATAR_BYTES {
        return Err(format!("压缩后的头像超过 {} KiB", MAX_AVATAR_BYTES / 1024));
    }
    if !valid_normalized_avatar(&png) {
        return Err("头像规范化结果无效".to_owned());
    }
    Ok(png)
}

pub fn valid_normalized_avatar(png: &[u8]) -> bool {
    png.len() <= MAX_AVATAR_BYTES
        && image::load_from_memory_with_format(png, image::ImageFormat::Png).is_ok_and(|image| {
            image.width() == AVATAR_DIMENSION && image.height() == AVATAR_DIMENSION
        })
}

pub fn image_handle_from_png(png: &[u8], images: &mut Assets<Image>) -> Option<Handle<Image>> {
    let dynamic = image::load_from_memory_with_format(png, image::ImageFormat::Png).ok()?;
    Some(images.add(Image::from_dynamic(
        dynamic,
        true,
        RenderAssetUsages::default(),
    )))
}

pub fn interaction_cooldown_mask_image(width: u32, height: u32, fraction: f32) -> Image {
    let mut pixels = image::RgbaImage::new(width, height);
    let center = Vec2::new(width as f32 * 0.5, height as f32 * 0.5);
    let sweep = fraction.clamp(0.0, 1.0) * std::f32::consts::TAU;
    for (x, y, pixel) in pixels.enumerate_pixels_mut() {
        let offset = Vec2::new(x as f32 + 0.5, y as f32 + 0.5) - center;
        let angle = offset.x.atan2(-offset.y).rem_euclid(std::f32::consts::TAU);
        if fraction >= 1.0 || (fraction > 0.0 && angle <= sweep) {
            *pixel = image::Rgba([0, 0, 0, 158]);
        }
    }
    Image::from_dynamic(
        image::DynamicImage::ImageRgba8(pixels),
        true,
        RenderAssetUsages::default(),
    )
}
