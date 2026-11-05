use super::EggThrowerCount;
use crate::app::presentation::{TEXT, add_text};
use crate::app::runtime::UiAssets;
use crate::app::shell::{SocialUiAction, UiAction};
use bevy::picking::Pickable;
use bevy::prelude::*;
use leocard_game_common::PlayerId;
use leocard_protocol::PlayerInteractionKind;
use leocard_ui::{TextureButton, UiComposer, sliced_image};

pub(crate) fn add_egg_thrower_button(
    commands: &mut UiComposer,
    parent: Entity,
    target: PlayerId,
    assets: &UiAssets,
) {
    let button = TextureButton {
        node: Node {
            width: px(69),
            height: px(64),
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            row_gap: px(2),
            ..default()
        },
        normal: sliced_image(assets.home.button.clone(), 32.0, 0.5),
        hovered: sliced_image(assets.home.purple_button_compact.clone(), 32.0, 0.5),
        action: Some(UiAction::Social(SocialUiAction::UseAutoEggThrower(target))),
        label: None,
        font_size: 12.0,
        color: TEXT,
        icon: None,
    }
    .mount(commands, parent, &assets.font)
    .0;
    add_egg_thrower_icon(commands, button, 36.0, assets);
    let count = add_text(commands, button, "砸蛋×0", 12.0, TEXT, assets);
    commands.entity(count).insert(EggThrowerCount);
}

pub(crate) fn add_egg_thrower_icon(
    commands: &mut UiComposer,
    parent: Entity,
    size: f32,
    assets: &UiAssets,
) -> Entity {
    let icon = commands.child(
        parent,
        (
            Node {
                width: px(size),
                height: px(size),
                min_width: px(size),
                min_height: px(size),
                position_type: PositionType::Relative,
                ..default()
            },
            Pickable::IGNORE,
        ),
    );
    let egg = assets.social.interaction_images[&(PlayerInteractionKind::Egg, false)].clone();
    for (index, angle) in [-33.0, -11.0, 11.0, 33.0].into_iter().enumerate() {
        commands.child(
            icon,
            (
                Node {
                    position_type: PositionType::Absolute,
                    left: px(size * (0.02 + index as f32 * 0.18)),
                    top: px(size * if index == 0 || index == 3 { 0.25 } else { 0.14 }),
                    width: px(size * 0.42),
                    height: px(size * 0.54),
                    ..default()
                },
                ImageNode {
                    // Trim the flight texture's transparent border for a readable small fan.
                    rect: Some(Rect::new(32.0, 24.0, 89.0, 94.0)),
                    ..ImageNode::new(egg.clone())
                },
                UiTransform::from_rotation(Rot2::degrees(angle)),
                Pickable::IGNORE,
            ),
        );
    }
    icon
}
