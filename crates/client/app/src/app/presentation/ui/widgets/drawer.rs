use crate::app::presentation::{ACCENT, GAME_SIDE_CONTROL_SCALE, TEXT};
use crate::app::runtime::UiAssets;
use crate::app::shell::UiAction;
use bevy::prelude::*;
use leocard_ui::{SwitchDrawer, SwitchDrawerStyle, UiComposer, spawn_switch_drawer};

pub(crate) fn add_switch_drawer(
    commands: &mut UiComposer,
    viewport: Entity,
    spec: SwitchDrawer<'_, UiAction>,
    assets: &UiAssets,
) -> Entity {
    let toggle = assets.tabletop.fold_image(!spec.expanded);
    spawn_switch_drawer(
        commands,
        viewport,
        spec,
        &SwitchDrawerStyle {
            scale: GAME_SIDE_CONTROL_SCALE,
            font: &assets.title_font,
            background: assets.tabletop.drawer_image(),
            checkbox: assets.home.checkbox.clone(),
            checkbox_selected: assets.home.checkbox_selected.clone(),
            toggle: toggle.clone(),
            toggle_hovered: toggle.with_color(Color::srgb(1.25, 1.25, 1.25)),
            active: ACCENT,
            muted: TEXT,
        },
    )
}
