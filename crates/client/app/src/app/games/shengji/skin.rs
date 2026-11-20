use crate::app::presentation::{
    PaperCardButtonTone, TableActionButton, TableButtonSkin, game_side_control_scale,
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
    action: Option<UiAction>,
    assets: &UiAssets,
    width: f32,
    height: f32,
    tone: PaperCardButtonTone,
) {
    TableActionButton {
        size: Vec2::new(width, height),
        ..TableActionButton::new(label, action, TableButtonSkin::PaperCard(tone))
    }
    .scaled(game_side_control_scale())
    .mount(commands, parent, assets);
}
