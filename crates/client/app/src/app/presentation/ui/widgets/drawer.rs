use crate::app::presentation::{ACCENT, MUTED};
use crate::app::runtime::UiAssets;
use crate::app::shell::UiAction;
use bevy::prelude::*;
use leocard_ui::{SwitchDrawer, SwitchDrawerStyle, UiComposer, sliced_image, spawn_switch_drawer};

pub(crate) fn add_switch_drawer(
    commands: &mut UiComposer,
    viewport: Entity,
    spec: SwitchDrawer<'_, UiAction>,
    assets: &UiAssets,
) -> Entity {
    spawn_switch_drawer(
        commands,
        viewport,
        spec,
        &SwitchDrawerStyle {
            font: &assets.font,
            background: sliced_image(assets.home.panel.clone(), 80.0, 0.20),
            checkbox: assets.home.checkbox.clone(),
            checkbox_selected: assets.home.checkbox_selected.clone(),
            left: assets.home.rule_left.clone(),
            left_hovered: assets.home.rule_left_highlighted.clone(),
            right: assets.home.rule_right.clone(),
            right_hovered: assets.home.rule_right_highlighted.clone(),
            active: ACCENT,
            muted: MUTED,
        },
    )
}
