//! 玩家头像装饰资源，与牌桌皮肤独立。

use bevy::image::{ImageLoaderSettings, ImageSampler};
use bevy::prelude::*;

#[derive(Default)]
pub(crate) struct AvatarFrameAssets {
    pub default: Handle<Image>,
}

impl AvatarFrameAssets {
    pub(crate) fn load(server: &AssetServer) -> Self {
        Self {
            default: server
                .load_builder()
                .with_settings(|settings: &mut ImageLoaderSettings| {
                    settings.sampler = ImageSampler::linear();
                })
                .load("ui/avatar-frames/avatar-frame.png"),
        }
    }
}
