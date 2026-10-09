use super::super::{QiGui523UiAction, QiGui523UiState};
use crate::app::presentation::{
    ACCENT, CardCounterCell, DrawerSwitch, SwitchDrawer, TEXT, add_card_counter_window,
    add_switch_drawer,
};
use crate::app::runtime::UiAssets;
use crate::app::shell::UiAction;
use bevy::prelude::*;

pub(in super::super) fn render_counter(
    commands: &mut Commands,
    table: Entity,
    ui: &QiGui523UiState,
    assets: &UiAssets,
) {
    let state = &ui.counter;
    if !state.window.active {
        return;
    }
    add_switch_drawer(
        commands,
        table,
        SwitchDrawer {
            expanded: state.window.drawer_open,
            toggle_action: UiAction::QiGui523(QiGui523UiAction::ToggleCounterDrawer),
            switches: &[DrawerSwitch {
                short: "牌",
                label: "记牌器",
                enabled: state.window.enabled,
                action: UiAction::QiGui523(QiGui523UiAction::ToggleCounter),
            }],
        },
        assets,
    );
    if !state.window.enabled {
        return;
    }
    let rows = state.counts.rows().map(|row| {
        row.map(|cell| {
            cell.map(|cell| CardCounterCell {
                image: assets.playing_cards.cards[&(cell.card.rank(), cell.card.suit())].clone(),
                remaining: cell.remaining,
                color: if cell.card.score() > 0 { ACCENT } else { TEXT },
            })
        })
    });
    add_card_counter_window::<QiGui523UiState>(commands, table, &state.window, rows, assets);
}
