//! Build-time bundled asset source used by release and explicit single-binary builds.

use super::*;

#[cfg(leocard_embedded_assets)]
include!(concat!(env!("OUT_DIR"), "/embedded_runtime_assets.rs"));

/// Register the in-memory directory as Bevy's default source before `AssetPlugin` is built.
/// An explicit `BEVY_ASSET_ROOT` keeps the filesystem source available for asset iteration and
/// emergency overrides, even in an otherwise self-contained release binary.
#[cfg(leocard_embedded_assets)]
pub(super) fn configure_runtime_asset_source(app: &mut App) {
    if std::env::var_os("BEVY_ASSET_ROOT").is_none() {
        use bevy::asset::AssetApp;
        use bevy::asset::io::memory::MemoryAssetReader;
        use bevy::asset::io::{AssetSourceBuilder, AssetSourceId};

        debug_assert!(EMBEDDED_ASSET_COUNT > 0);
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
pub(super) fn configure_runtime_asset_source(_app: &mut App) {}

#[cfg(all(test, leocard_embedded_assets))]
mod tests {
    use super::*;

    #[test]
    fn embedded_bundle_contains_representative_runtime_assets() {
        let directory = embedded_asset_dir();
        assert!(EMBEDDED_ASSET_COUNT > 100);
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
