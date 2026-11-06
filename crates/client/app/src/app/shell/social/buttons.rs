use super::{InteractionCooldownMask, SocialUiAction, add_shoe_button};
use crate::app::presentation::{GAME_SIDE_CONTROL_SCALE, TEXT};
use crate::app::runtime::UiAssets;
use crate::app::shell::UiAction;
use bevy::picking::Pickable;
use bevy::prelude::*;
use leocard_game_common::PlayerId;
use leocard_protocol::PlayerInteractionKind;
use leocard_ui::{TextureButton, UiComposer, sliced_image, spawn_node};

pub(super) fn interaction_button<'a, A>(
    assets: &UiAssets,
    action: A,
    label: Option<&'a str>,
    icon: Option<Handle<Image>>,
) -> TextureButton<'a, A> {
    TextureButton {
        node: Node {
            width: px(69.0 * GAME_SIDE_CONTROL_SCALE),
            height: px(64.0 * GAME_SIDE_CONTROL_SCALE),
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            row_gap: px(2.0 * GAME_SIDE_CONTROL_SCALE),
            ..default()
        },
        normal: sliced_image(assets.home.button.clone(), 32.0, 0.5),
        hovered: sliced_image(assets.home.purple_button_compact.clone(), 32.0, 0.5),
        action: Some(action),
        label,
        font_size: 12.0 * GAME_SIDE_CONTROL_SCALE,
        color: TEXT,
        icon: icon.map(|image| (image, 36.0 * GAME_SIDE_CONTROL_SCALE)),
    }
}

pub(crate) fn add_interaction_buttons(
    commands: &mut UiComposer,
    parent: Entity,
    target: PlayerId,
    assets: &UiAssets,
) {
    let actions = spawn_node(
        commands,
        parent,
        Node {
            width: percent(100),
            height: px(64.0 * GAME_SIDE_CONTROL_SCALE),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            column_gap: px(0),
            ..default()
        },
        None,
    );
    for (kind, label) in [
        (PlayerInteractionKind::Flower, "鲜花"),
        (PlayerInteractionKind::Egg, "鸡蛋"),
        (PlayerInteractionKind::Wine, "酒杯"),
        (PlayerInteractionKind::Shoe, "拖鞋"),
    ] {
        let icon = assets
            .social
            .interaction_images
            .get(&(kind, false))
            .expect("every interaction has a flight image")
            .clone();
        let button = if kind == PlayerInteractionKind::Shoe {
            add_shoe_button(commands, actions, target, assets)
        } else {
            interaction_button(
                assets,
                UiAction::Social(SocialUiAction::SendInteraction { target, kind }),
                Some(label),
                Some(icon),
            )
            .mount(commands, actions, &assets.font)
            .0
        };
        if matches!(
            kind,
            PlayerInteractionKind::Wine | PlayerInteractionKind::Shoe
        ) {
            commands.child(
                button,
                (
                    Node {
                        position_type: PositionType::Absolute,
                        left: px(0),
                        right: px(0),
                        top: px(0),
                        bottom: px(0),
                        ..default()
                    },
                    ImageNode::new(
                        assets
                            .social
                            .interaction_cooldown_masks
                            .last()
                            .cloned()
                            .unwrap_or_default(),
                    ),
                    InteractionCooldownMask {
                        player: target,
                        kind,
                    },
                    Visibility::Hidden,
                    ZIndex(10),
                    Pickable::IGNORE,
                ),
            );
        }
    }
}
