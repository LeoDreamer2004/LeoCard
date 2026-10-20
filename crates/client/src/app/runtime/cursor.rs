use bevy::asset::RenderAssetUsages;
use bevy::prelude::*;
use bevy::window::{CursorIcon, CustomCursor, CustomCursorImage, PrimaryWindow};
use image::imageops::FilterType;
use image::{DynamicImage, ImageFormat};
use std::collections::HashMap;

pub(crate) struct CursorPlugin;

impl Plugin for CursorPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CursorTexture>()
            .add_systems(Update, update_cursor);
    }
}

#[derive(Resource)]
struct CursorTexture {
    source: DynamicImage,
    variants: HashMap<u32, CursorIcon>,
    applied: Option<(Entity, u32)>,
}

impl Default for CursorTexture {
    fn default() -> Self {
        Self {
            source: image::load_from_memory_with_format(
                include_bytes!("../../../../../assets/icons/cursor-lavender.png"),
                ImageFormat::Png,
            )
            .expect("the compiled cursor texture must be a valid PNG"),
            variants: HashMap::new(),
            applied: None,
        }
    }
}

impl CursorTexture {
    fn icon(&mut self, size: u32, images: &mut Assets<Image>) -> CursorIcon {
        self.variants
            .entry(size)
            .or_insert_with(|| {
                // The source includes transparent padding; 48 px gives a roughly 40 px arrow.
                let pixels = self.source.resize_exact(size, size, FilterType::Nearest);
                let hotspot = (
                    (237.0 * size as f32 / self.source.width() as f32).round() as u16,
                    (118.0 * size as f32 / self.source.height() as f32).round() as u16,
                );
                CursorIcon::Custom(CustomCursor::Image(CustomCursorImage {
                    handle: images.add(Image::from_dynamic(
                        pixels,
                        true,
                        RenderAssetUsages::MAIN_WORLD,
                    )),
                    hotspot,
                    ..default()
                }))
            })
            .clone()
    }
}

fn update_cursor(
    mut commands: Commands,
    window: Single<(Entity, &Window), With<PrimaryWindow>>,
    mut texture: ResMut<CursorTexture>,
    mut images: ResMut<Assets<Image>>,
) {
    let (entity, window) = *window;
    // Winit uses logical image dimensions for NSCursor, physical pixels elsewhere.
    let scale = if cfg!(target_os = "macos") {
        1.0
    } else {
        window.scale_factor()
    };
    let size = (48.0 * scale).round().clamp(24.0, 256.0) as u32;
    if texture.applied == Some((entity, size)) {
        return;
    }

    let icon = texture.icon(size, &mut images);
    commands.entity(entity).insert(icon);
    texture.applied = Some((entity, size));
}
