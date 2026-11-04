//! 内嵌资源源与操作系统窗口集成。

use bevy::asset::AssetApp;
use bevy::asset::io::memory::{Dir, MemoryAssetReader};
use bevy::asset::io::{AssetSourceBuilder, AssetSourceId};
#[cfg(not(target_os = "android"))]
use bevy::ecs::system::NonSendMarker;
use bevy::prelude::*;
#[cfg(not(target_os = "android"))]
use bevy::window::WindowCreated;
#[cfg(not(target_os = "android"))]
use bevy::winit::WINIT_WINDOWS;
use std::env;
use std::path::Path;
#[cfg(target_os = "windows")]
use winit::platform::windows::WindowExtWindows;
#[cfg(not(target_os = "android"))]
use winit::window::Icon;

#[cfg(not(target_os = "android"))]
pub(super) const APP_ICON_PNG: &[u8] =
    include_bytes!("../../../../../../assets/icons/app-icon.png");

fn embedded_asset_dir() -> Dir {
    let directory = Dir::default();
    for &(path, bytes) in leocard_assets::embedded_assets() {
        directory.insert_asset(Path::new(path), bytes);
    }
    directory
}

/// 在构造 `AssetPlugin` 前将编译进程序的目录注册成 Bevy 默认资源源。
/// 显式指定 `BEVY_ASSET_ROOT` 时仍使用文件系统，便于开发和紧急覆盖。
pub fn configure_runtime_asset_source(app: &mut App) {
    if leocard_assets::IS_EMBEDDED && env::var_os("BEVY_ASSET_ROOT").is_none() {
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

#[cfg(not(target_os = "android"))]
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

#[cfg(not(target_os = "android"))]
fn decode_app_icon() -> Icon {
    let pixels = image::load_from_memory(APP_ICON_PNG)
        .expect("the compiled application icon must be a valid PNG")
        .thumbnail(256, 256)
        .into_rgba8();
    let (width, height) = pixels.dimensions();
    Icon::from_rgba(pixels.into_raw(), width, height)
        .expect("the compiled application icon must contain RGBA pixels")
}

#[cfg(target_os = "android")]
pub(super) fn set_app_window_icon() {}
