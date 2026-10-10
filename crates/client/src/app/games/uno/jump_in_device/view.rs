use super::super::UnoUiAction;
use super::UnoJumpInDevice;
use crate::app::presentation::{DrawerSwitch, SwitchDrawer, add_switch_drawer};
use crate::app::runtime::UiAssets;
use crate::app::shell::UiAction;
use bevy::prelude::*;

pub(in super::super) fn render_jump_in_device(
    commands: &mut Commands,
    content: Entity,
    device: &UnoJumpInDevice,
    assets: &UiAssets,
) {
    if !device.available {
        return;
    }
    let switches = [DrawerSwitch {
        short: "抢",
        label: "自动抢出",
        enabled: device.enabled,
        action: UiAction::Uno(UnoUiAction::ToggleJumpInDevice),
    }];
    add_switch_drawer(
        commands,
        content,
        SwitchDrawer {
            expanded: device.drawer_open,
            toggle_action: UiAction::Uno(UnoUiAction::ToggleJumpInDrawer),
            switches: &switches,
        },
        assets,
    );
}
