use crate::app::presentation::{GameButtonSpec, GameButtonTone, add_textured_game_button};
use crate::app::runtime::UiAssets;
use crate::app::shell::UiAction;
use bevy::prelude::*;

#[expect(
    clippy::too_many_arguments,
    reason = "button label, action, appearance and dimensions remain explicit"
)]
pub(super) fn add_shengji_button(
    commands: &mut Commands,
    parent: Entity,
    label: &str,
    action: UiAction,
    assets: &UiAssets,
    width: f32,
    height: f32,
    tone: GameButtonTone,
) {
    add_textured_game_button(
        commands,
        parent,
        assets,
        GameButtonSpec {
            width,
            height,
            ..GameButtonSpec::action(assets, label, Some(action), tone)
        },
    );
}

pub(super) fn add_shengji_disabled_button(
    commands: &mut Commands,
    parent: Entity,
    label: &str,
    assets: &UiAssets,
    width: f32,
    height: f32,
) {
    add_textured_game_button(
        commands,
        parent,
        assets,
        GameButtonSpec {
            width,
            height,
            ..GameButtonSpec::action(assets, label, None, GameButtonTone::Play)
        },
    );
}
