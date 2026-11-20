//! 内嵌和网页分包的内存资源源与操作系统窗口集成。

use bevy::asset::AssetApp;
#[cfg(not(feature = "web"))]
use bevy::asset::io::memory::Dir;
use bevy::asset::io::memory::MemoryAssetReader;
use bevy::asset::io::{AssetSourceBuilder, AssetSourceId};
#[cfg(feature = "desktop")]
use bevy::ecs::system::NonSendMarker;
use bevy::prelude::*;
#[cfg(feature = "desktop")]
use bevy::window::WindowCreated;
#[cfg(feature = "desktop")]
use bevy::winit::WINIT_WINDOWS;
#[cfg(not(feature = "web"))]
use std::env;
#[cfg(not(feature = "web"))]
use std::path::Path;
#[cfg(target_os = "windows")]
use winit::platform::windows::WindowExtWindows;
#[cfg(feature = "desktop")]
use winit::window::Icon;

#[cfg(feature = "desktop")]
pub(super) const APP_ICON_PNG: &[u8] =
    include_bytes!("../../../../../../assets/icons/app-icon.png");

#[cfg(not(feature = "web"))]
fn embedded_asset_dir() -> Dir {
    let directory = Dir::default();
    for &(path, bytes) in leocard_assets::embedded_assets() {
        directory.insert_asset(Path::new(path), bytes);
    }
    directory
}

/// 在构造 `AssetPlugin` 前注册内嵌或浏览器解包得到的内存资源。
/// 原生平台显式指定 `BEVY_ASSET_ROOT` 时仍使用文件系统。
pub fn configure_runtime_asset_source(app: &mut App) {
    #[cfg(feature = "web")]
    let directory = Some(super::web::asset_directory());
    #[cfg(not(feature = "web"))]
    let directory = (leocard_assets::IS_EMBEDDED && env::var_os("BEVY_ASSET_ROOT").is_none())
        .then(embedded_asset_dir);
    if let Some(directory) = directory {
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

#[cfg(feature = "desktop")]
pub(super) fn set_app_window_icon(
    mut created_windows: MessageReader<WindowCreated>,
    _main_thread: NonSendMarker,
) {
    for event in created_windows.read() {
        let icon = decode_app_icon();
        WINIT_WINDOWS.with_borrow(|windows| {
            if let Some(window) = windows.get_window(event.window) {
                #[cfg(target_os = "windows")]
                window.set_taskbar_icon(Some(icon.clone()));
                window.set_window_icon(Some(icon));
            }
        });
    }
}

#[cfg(feature = "desktop")]
fn decode_app_icon() -> Icon {
    let pixels = image::load_from_memory(APP_ICON_PNG)
        .expect("the compiled application icon must be a valid PNG")
        .thumbnail(256, 256)
        .into_rgba8();
    let (width, height) = pixels.dimensions();
    Icon::from_rgba(pixels.into_raw(), width, height)
        .expect("the compiled application icon must contain RGBA pixels")
}

#[cfg(not(feature = "desktop"))]
pub(super) fn set_app_window_icon() {}
