use super::super::{TexasTableLayout, TexasUiAction};
use crate::app::presentation::{CasinoButtonTone, TableActionButton, TableButtonSkin};
use crate::app::runtime::UiAssets;
use crate::app::shell::UiAction;
use bevy::prelude::*;
use leocard_ui::UiComposer;

pub(super) fn add_texas_action_button(
    commands: &mut UiComposer,
    parent: Entity,
    label: &str,
    action: TexasUiAction,
    tone: CasinoButtonTone,
    assets: &UiAssets,
) -> Entity {
    let scale = TexasTableLayout::ACTION_SCALE;
    TableActionButton {
        size: Vec2::new(118.0, 50.0) * scale,
        font_size: 26.0 * scale,
        ..TableActionButton::new(
            label,
            Some(UiAction::Texas(action)),
            TableButtonSkin::Casino(tone),
        )
    }
    .mount(commands, parent, assets)
    .0
}

pub(super) fn add_texas_raise_adjust_button(
    commands: &mut UiComposer,
    parent: Entity,
    label: &str,
    action: TexasUiAction,
    assets: &UiAssets,
) -> Entity {
    TableActionButton {
        size: Vec2::splat(38.0),
        font_size: 17.0,
        ..TableActionButton::new(
            label,
            Some(UiAction::Texas(action)),
            TableButtonSkin::Casino(CasinoButtonTone::Call),
        )
    }
    .scaled(TexasTableLayout::ACTION_SCALE)
    .mount(commands, parent, assets)
    .0
}
