use super::super::{BORDER, TEXT};
use super::text_scene;
use crate::app::runtime::UiAssets;
use bevy::picking::Pickable;
use bevy::prelude::*;
use leocard_ui::{AvatarFrame, AvatarFrameEntities, UiComposer, UiSceneKey, ui_component};

#[track_caller]
pub(crate) fn add_avatar(
    commands: &mut UiComposer,
    parent: Entity,
    name: &str,
    image: Option<&Handle<Image>>,
    size: f32,
    assets: &UiAssets,
) -> Entity {
    commands.node(
        parent,
        bsn! {
            Node {
                width: px(size), height: px(size), min_width: px(size),
                border: UiRect::all(px(1)), border_radius: BorderRadius::all(percent(50)),
                align_items: AlignItems::Center, justify_content: JustifyContent::Center,
            }
            BorderColor::all(BORDER)
            @avatar_content(name, image, size, assets)
        },
    )
}

pub(crate) fn add_framed_avatar(
    commands: &mut UiComposer,
    parent: Entity,
    key: &str,
    name: &str,
    image: Option<&Handle<Image>>,
    size: f32,
    assets: &UiAssets,
) -> AvatarFrameEntities {
    AvatarFrame {
        texture: assets.tabletop.avatar_frame.clone(),
        size,
    }
    .mount(
        commands,
        parent,
        key,
        avatar_content(name, image, size, assets),
    )
}

fn avatar_content(
    name: &str,
    image: Option<&Handle<Image>>,
    size: f32,
    assets: &UiAssets,
) -> impl Scene {
    let picture = image.map(|image| ui_component(ImageNode::new(image.clone())));
    let initial = image.is_none().then(|| {
        let text = text_scene(
            assets,
            name.chars().next().unwrap_or('玩').to_string(),
            size * 0.42,
            TEXT,
        );
        bsn! {
            Children [ UiSceneKey("initial") @text Pickable::IGNORE ]
        }
    });
    bsn! {
        BackgroundColor({avatar_color(name)})
        @picture
        @initial
    }
}

pub(crate) fn avatar_color(name: &str) -> Color {
    const COLORS: [Color; 6] = [
        Color::srgb(0.20, 0.48, 0.76),
        Color::srgb(0.65, 0.29, 0.68),
        Color::srgb(0.82, 0.35, 0.25),
        Color::srgb(0.18, 0.62, 0.48),
        Color::srgb(0.76, 0.53, 0.16),
        Color::srgb(0.35, 0.42, 0.72),
    ];
    let hash = name
        .bytes()
        .fold(0_usize, |hash, byte| hash.wrapping_mul(31) + byte as usize);
    COLORS[hash % COLORS.len()]
}
