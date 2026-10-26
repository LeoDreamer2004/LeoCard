use super::super::{ShengjiUiAction, ShengjiUiState};
use crate::app::presentation::{
    ACCENT, CardCounterCell, TEXT, add_card_counter_window, add_switch_drawer,
};
use crate::app::runtime::UiAssets;
use crate::app::shell::UiAction;
use bevy::prelude::*;
use leocard_protocol::ShengjiSnapshot;
use leocard_ui::{DrawerSwitch, SwitchDrawer, UiComposer};

pub(in super::super) fn render_counter(
    commands: &mut UiComposer,
    root: Entity,
    table: Entity,
    game: &ShengjiSnapshot,
    ui: &ShengjiUiState,
    assets: &UiAssets,
) {
    let state = &ui.counter;
    if !state.window.active {
        return;
    }
    let drawer = add_switch_drawer(
        commands,
        root,
        SwitchDrawer {
            expanded: state.window.drawer_open,
            toggle_action: UiAction::Shengji(ShengjiUiAction::ToggleCounterDrawer),
            switches: &[DrawerSwitch {
                short: "牌",
                label: "记牌器",
                enabled: state.window.enabled,
                action: UiAction::Shengji(ShengjiUiAction::ToggleCounter),
            }],
        },
        assets,
    );
    // Dock to the full viewport, retaining the 82px clearance above the 210px hand area.
    commands
        .entity(drawer)
        .entry::<Node>()
        .and_modify(|mut node| {
            node.bottom = px(292);
        });
    if !state.window.enabled {
        return;
    }
    let Some(trump) = game.trump else {
        return;
    };
    let rows = state.counts.rows(trump).map(|row| {
        row.map(|cell| {
            cell.map(|cell| CardCounterCell {
                image: assets.playing_cards.face(cell.card),
                remaining: cell.remaining,
                color: if trump.is_trump(cell.card) {
                    ACCENT
                } else {
                    TEXT
                },
            })
        })
    });
    add_card_counter_window::<ShengjiUiState>(commands, table, &state.window, rows, assets);
}
