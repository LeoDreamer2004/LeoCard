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
    let (toggle, toggle_hovered) = if spec.expanded {
        (&assets.home.rule_left, &assets.home.rule_left_highlighted)
    } else {
        (&assets.home.rule_right, &assets.home.rule_right_highlighted)
    };
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
            toggle: ImageNode::new(toggle.clone()),
            toggle_hovered: ImageNode::new(toggle_hovered.clone()),
            active: ACCENT,
            muted: TEXT,
        },
    )
}
