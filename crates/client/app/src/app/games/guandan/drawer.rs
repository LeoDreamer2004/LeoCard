use super::actions::GuandanUiAction;
use super::arrangement::HandLayoutMode;
use super::state::GuandanUiState;
use crate::app::presentation::add_switch_drawer;
use crate::app::runtime::UiAssets;
use crate::app::shell::UiAction;
use bevy::prelude::*;
use leocard_ui::{DrawerSwitch, SwitchDrawer, UiComposer};

pub(super) fn render(
    commands: &mut UiComposer,
    viewport: Entity,
    ui: &GuandanUiState,
    assets: &UiAssets,
) {
    let mut switches = vec![DrawerSwitch {
        short: "竖",
        label: "竖叠手牌",
        enabled: ui.hand_layout == HandLayoutMode::Columns,
        action: UiAction::Guandan(GuandanUiAction::ToggleHandLayout),
    }];
    if ui.counter.active {
        switches.push(DrawerSwitch {
            short: "牌",
            label: "记牌器",
            enabled: ui.counter.enabled,
            action: UiAction::Guandan(GuandanUiAction::ToggleCounter),
        });
    }
    let drawer = add_switch_drawer(
        commands,
        viewport,
        SwitchDrawer {
            expanded: ui.drawer_open,
            toggle_action: UiAction::Guandan(GuandanUiAction::ToggleDrawer),
            switches: &switches,
        },
        assets,
    );
    commands
        .entity(drawer)
        .entry::<Node>()
        .and_modify(|mut node| {
            node.bottom = px(150);
        });
}
