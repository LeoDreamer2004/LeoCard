use super::bomb::add_bomb_play_effect;
use super::heaven_bomb::add_heaven_bomb_effect;
use super::state::{PlayEffectKind, PlayEffectRoot};
use crate::app::runtime::UiAssets;
use bevy::picking::Pickable;
use bevy::prelude::*;
use leocard_ui::{UiComposer, spawn_node};

pub(crate) fn add_play_effect_overlay(
    commands: &mut UiComposer,
    table: Entity,
    full_screen_parent: Entity,
    kind: PlayEffectKind,
    source: Vec2,
    assets: &UiAssets,
) {
    let parent = if matches!(kind, PlayEffectKind::HeavenBomb) {
        full_screen_parent
    } else {
        table
    };
    let root = spawn_node(
        commands,
        parent,
        Node {
            position_type: PositionType::Absolute,
            left: px(0),
            right: px(0),
            top: px(0),
            bottom: px(0),
            ..default()
        },
        None,
    );
    commands.entity(root).insert((
        PlayEffectRoot,
        Visibility::Hidden,
        GlobalZIndex(if matches!(kind, PlayEffectKind::HeavenBomb) {
            1900
        } else {
            1100
        }),
        Pickable::IGNORE,
    ));
    match kind {
        PlayEffectKind::HeavenBomb => add_heaven_bomb_effect(commands, root, assets),
        PlayEffectKind::Bomb => {
            add_bomb_play_effect(commands, root, source);
        }
        _ => {}
    }
}
