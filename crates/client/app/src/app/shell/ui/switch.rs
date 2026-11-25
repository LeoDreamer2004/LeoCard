use crate::app::presentation::READY;
use crate::app::runtime::UiAssets;
use crate::app::shell::UiAction;
use bevy::prelude::*;
use leocard_ui::{SwitchStyle, UiComposer, add_switch, sliced_image};

pub(crate) fn add_cozy_switch(
    commands: &mut UiComposer,
    parent: Entity,
    selected: bool,
    action: Option<UiAction>,
    assets: &UiAssets,
) -> Entity {
    add_switch(
        commands,
        parent,
        selected,
        action,
        SwitchStyle {
            track: sliced_image(assets.home.input.clone(), 32.0, 0.35),
            thumb: sliced_image(assets.home.button.clone(), 32.0, 0.32),
            off_color: Color::srgb(0.84, 0.84, 0.88),
            on_color: READY,
            disabled_mask: Color::srgba(0.42, 0.42, 0.45, 0.68),
            scale: 1.0,
        },
    )
}
