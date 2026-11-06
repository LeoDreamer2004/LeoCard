use super::super::buttons::interaction_button;
use super::EggThrowerCount;
use crate::app::presentation::{GAME_SIDE_CONTROL_SCALE, TEXT, add_text};
use crate::app::runtime::UiAssets;
use crate::app::shell::{SocialUiAction, UiAction};
use bevy::picking::Pickable;
use bevy::prelude::*;
use leocard_game_common::PlayerId;
use leocard_protocol::PlayerInteractionKind;
use leocard_ui::UiComposer;

pub(crate) fn add_egg_thrower_button(
    commands: &mut UiComposer,
    parent: Entity,
    target: PlayerId,
    assets: &UiAssets,
) -> Entity {
    let mut spec = interaction_button(
        assets,
        UiAction::Social(SocialUiAction::UseAutoEggThrower(target)),
        None,
        None,
    );
    spec.node.position_type = PositionType::Absolute;
    spec.node.top = percent(100);
    spec.node.margin = UiRect::top(px(6.0 * GAME_SIDE_CONTROL_SCALE));
    let button = spec.mount(commands, parent, &assets.font).0;
    add_egg_thrower_icon(commands, button, 36.0 * GAME_SIDE_CONTROL_SCALE, assets);
    let count = add_text(
        commands,
        button,
        "砸蛋×0",
        12.0 * GAME_SIDE_CONTROL_SCALE,
        TEXT,
        assets,
    );
    commands.entity(count).insert(EggThrowerCount);
    button
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
