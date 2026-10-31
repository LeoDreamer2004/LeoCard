//! 内嵌资源源与操作系统窗口集成。

#[cfg(test)]
use super::{TABLE_FELT_ASSET, UI_FONT_ASSET};
#[cfg(test)]
use crate::app::presentation::TABLE_BACKGROUND_SHADER;
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn embedded_bundle_contains_representative_runtime_assets() {
        if !leocard_assets::IS_EMBEDDED {
            return;
        }
        let directory = embedded_asset_dir();
        for path in [
            UI_FONT_ASSET,
            TABLE_FELT_ASSET,
            TABLE_BACKGROUND_SHADER,
            "shaders/uno_palette.wesl",
            "cards/uno/card_back.png",
            "cards/uno/wild.png",
            "cards/uno-extension/uno-flip/dark/purple_flip.png",
            "cards/uno-extension/uno-flip/light/wild_draw_two.png",
            "vendor/kenney/boardgame/PNG/Cards/cardSpadesA.png",
            "vendor/kenney/interface-sounds/Audio/error_007.ogg",
            "vendor/noname/voice/male/22.mp3",
            "vendor/noname/voice/female/22.mp3",
            "ui/home/cozy-panel.png",
            "ui/home/cozy-game-card.png",
            "ui/home/cozy-game-card-hover.png",
            "ui/game-buttons/warning-normal.png",
            "ui/minecraft_guess/icons/entries.png",
            "ui/minecraft_guess/background/block-fronts.png",
            "ui/minecraft_guess/background/dirt.png",
            "ui/minecraft_guess/background/panoramas/1.21.png",
            "shaders/minecraft_background.wesl",
            "ui/game-buttons/adjust-left-normal.png",
            "ui/game-buttons/adjust-right-hover.png",
            "ui/home/cozy-button-purple-plain.png",
            "ui/home/cozy-button-arrows.png",
            "ui/home/cozy-button-compact.png",
            "ui/home/cozy-button-danger-compact.png",
            "ui/home/cozy-button-danger-hover-compact.png",
            "ui/home/cozy-button-cool-compact.png",
            "ui/home/cozy-button-cool-hover-compact.png",
            "ui/home/cozy-button-purple-compact.png",
            "ui/home/cozy-input-compact.png",
            "ui/home/cozy-input-focused-compact.png",
            "icons/github-mark.png",
            "icons/settings.png",
            "icons/achievement-trophy.png",
            "ui/achievements/emblem-mahjong.png",
            "ui/achievements/medal-gold.png",
            "icons/exit-room.png",
        ] {
            assert!(
                directory.get_asset(Path::new(path)).is_some(),
                "missing embedded runtime asset: {path}"
            );
        }
    }
}

#[cfg(target_os = "android")]
pub(super) fn set_app_window_icon() {}
