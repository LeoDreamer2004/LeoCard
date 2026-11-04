//! 玩家头像装饰资源，与牌桌皮肤独立。

use bevy::image::{ImageLoaderSettings, ImageSampler};
use bevy::prelude::*;
use leocard_client::ShopAvatarFrame;
use leocard_protocol::AvatarFrameId;
use leocard_ui::{AvatarFrame, AvatarFrameLayout};
use std::collections::BTreeMap;

const TEXTURE_SIZE: f32 = 128.0;
// The straight rails of walnut.png sit one pixel in from its canvas.
const RAIL_INSET: f32 = 1.0;
const RAIL_SIZE: f32 = TEXTURE_SIZE - 2.0 * RAIL_INSET;
const PORTRAIT_OVERLAP: f32 = 2.0;
const EXPORT_PADDING: f32 = 16.0 / TEXTURE_SIZE;

#[derive(Default)]
pub(crate) struct AvatarFrameAssets {
    textures: BTreeMap<AvatarFrameId, Handle<Image>>,
}

impl AvatarFrameAssets {
    pub(crate) fn load(server: &AssetServer) -> Self {
        Self {
            textures: ShopAvatarFrame::ALL
                .into_iter()
                .map(|frame| {
                    let texture = server
                        .load_builder()
                        .with_settings(|settings: &mut ImageLoaderSettings| {
                            settings.sampler = ImageSampler::linear();
                        })
                        .load(format!("ui/avatar-frames/{}.png", frame.asset));
                    (frame.id, texture)
                })
                .collect(),
        }
    }

    pub(crate) fn frame(&self, id: AvatarFrameId, size: f32) -> AvatarFrame {
        AvatarFrame {
            texture: self.textures.get(&id).cloned().unwrap_or_default(),
            size,
            layout: frame_layout(id),
        }
    }
}

fn frame_layout(id: AvatarFrameId) -> AvatarFrameLayout {
    // Source pixel bounds of the straight outer rails and the clear inner opening.
    // Corner ornaments intentionally extend past these measurements.
    let (rails, opening): ([u8; 4], [u8; 4]) = match id {
        AvatarFrameId::Default => return AvatarFrameLayout::default(),
        AvatarFrameId::QqAdmin => ([10, 13, 115, 118], [18, 22, 106, 109]),
        AvatarFrameId::IronBasin => ([8, 13, 116, 119], [15, 21, 109, 111]),
        AvatarFrameId::ChinaRose => ([9, 11, 116, 118], [17, 19, 108, 110]),
        AvatarFrameId::Cattail => ([8, 10, 120, 122], [14, 16, 113, 115]),
        AvatarFrameId::Study => ([9, 12, 119, 121], [16, 18, 111, 114]),
        AvatarFrameId::Create => ([11, 11, 116, 117], [18, 19, 109, 109]),
        AvatarFrameId::GregTech => ([8, 8, 119, 121], [18, 18, 109, 108]),
        AvatarFrameId::Orange => ([9, 13, 116, 120], [15, 21, 109, 111]),
        AvatarFrameId::Horse => ([10, 15, 116, 122], [16, 21, 109, 116]),
        AvatarFrameId::Host => ([16, 13, 113, 112], [23, 21, 106, 104]),
        AvatarFrameId::WhaleGirl => ([8, 10, 114, 117], [14, 16, 108, 111]),
    };
    let [left, top, right, bottom] = rails.map(f32::from);
    let [inner_left, inner_top, inner_right, inner_bottom] = opening.map(f32::from);
    let source_origin = Vec2::new(left, top);
    let scale = Vec2::splat(RAIL_SIZE) / (Vec2::new(right, bottom) - source_origin);
    let place = |point| (point - source_origin) * scale + Vec2::splat(RAIL_INSET);
    let overlap = Vec2::splat(PORTRAIT_OVERLAP);
    AvatarFrameLayout {
        // The high-resolution originals are downsampled directly onto a 160px canvas.
        // A 16px surround gives the ornaments room beyond the 128px frame area.
        artwork: Rect::new(
            -EXPORT_PADDING,
            -EXPORT_PADDING,
            1.0 + EXPORT_PADDING,
            1.0 + EXPORT_PADDING,
        ),
        portrait: Rect {
            min: (place(Vec2::new(inner_left, inner_top)) - overlap) / TEXTURE_SIZE,
            max: (place(Vec2::new(inner_right, inner_bottom)) + overlap) / TEXTURE_SIZE,
        },
    }
}
