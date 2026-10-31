use crate::app::presentation::{
    GAME_SIDE_CONTROL_SCALE, GameButtonSpec, GameButtonTone, add_textured_game_button,
};
use crate::app::runtime::UiAssets;
use crate::app::shell::UiAction;
use bevy::prelude::*;
use leocard_ui::UiComposer;

#[expect(
    clippy::too_many_arguments,
    reason = "button label, action, appearance and dimensions remain explicit"
)]
pub(super) fn add_shengji_button(
    commands: &mut UiComposer,
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
            ..GameButtonSpec::compact(assets, label, Some(action), tone)
        }
        .scaled(GAME_SIDE_CONTROL_SCALE),
    );
}

pub(super) fn add_shengji_disabled_button(
    commands: &mut UiComposer,
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
            ..GameButtonSpec::compact(assets, label, None, GameButtonTone::Play)
        }
        .scaled(GAME_SIDE_CONTROL_SCALE),
    );
}
