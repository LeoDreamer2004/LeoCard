//! 内嵌资源源与操作系统窗口集成。

#[cfg(all(test, leocard_embedded_assets))]
use crate::app::{TABLE_BACKGROUND_SHADER, TABLE_FELT_ASSET, UI_FONT_ASSET};
#[cfg(leocard_embedded_assets)]
use bevy::asset::AssetApp;
#[cfg(leocard_embedded_assets)]
use bevy::asset::io::memory::MemoryAssetReader;
#[cfg(leocard_embedded_assets)]
use bevy::asset::io::{AssetSourceBuilder, AssetSourceId};
use bevy::prelude::*;
use bevy::window::WindowCreated;
#[cfg(all(test, leocard_embedded_assets))]
use std::path::Path;
use winit::window::Icon;

#[cfg(leocard_embedded_assets)]
include!(concat!(env!("OUT_DIR"), "/embedded_runtime_assets.rs"));

const APP_ICON_PNG: &[u8] = include_bytes!("../../../../../assets/icons/app-icon.png");

/// 在构造 `AssetPlugin` 前将编译进程序的目录注册成 Bevy 默认资源源。
/// 显式指定 `BEVY_ASSET_ROOT` 时仍使用文件系统，便于开发和紧急覆盖。
#[cfg(leocard_embedded_assets)]
pub fn configure_runtime_asset_source(app: &mut App) {
    if std::env::var_os("BEVY_ASSET_ROOT").is_none() {
        let directory = embedded_asset_dir();
        app.register_asset_source(
            AssetSourceId::Default,
            AssetSourceBuilder::new(move || {
                Box::new(MemoryAssetReader {
                    root: directory.clone(),
                })
            }),
        );
    }
}

#[cfg(not(leocard_embedded_assets))]
pub fn configure_runtime_asset_source(_app: &mut App) {}

pub fn set_app_window_icon(mut created_windows: MessageReader<WindowCreated>) {
    for event in created_windows.read() {
        let icon = decode_app_icon();
        bevy::winit::WINIT_WINDOWS.with_borrow(|windows| {
            if let Some(window) = windows.get_window(event.window) {
                window.set_window_icon(Some(icon));
            }
        });
    }
}

fn decode_app_icon() -> Icon {
    let pixels = image::load_from_memory(APP_ICON_PNG)
        .expect("the compiled application icon must be a valid PNG")
        .into_rgba8();
    let (width, height) = pixels.dimensions();
    Icon::from_rgba(pixels.into_raw(), width, height)
        .expect("the compiled application icon must contain RGBA pixels")
}

#[cfg(all(test, leocard_embedded_assets))]
mod tests {
    use super::*;

    #[test]
    fn embedded_bundle_contains_representative_runtime_assets() {
        let directory = embedded_asset_dir();
        for path in [
            UI_FONT_ASSET,
            TABLE_FELT_ASSET,
            TABLE_BACKGROUND_SHADER,
            "shaders/uno_palette.wgsl",
            "cards/uno/card_back.png",
            "cards/uno/wild.png",
            "cards/uno-extension/uno-flip/dark/purple_flip.png",
            "cards/uno-extension/uno-flip/light/wild_draw_two.png",
            "vendor/kenney/boardgame/PNG/Cards/cardSpadesA.png",
            "vendor/kenney/interface-sounds/Audio/error_007.ogg",
            "vendor/noname/voice/male/22.mp3",
            "ui/panel_window.png",
            "icons/github-mark.png",
        ] {
            assert!(
                directory.get_asset(Path::new(path)).is_some(),
                "missing embedded runtime asset: {path}"
            );
        }
    }
}
