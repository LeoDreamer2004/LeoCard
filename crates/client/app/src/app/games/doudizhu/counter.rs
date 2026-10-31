use super::{DoudizhuUiAction, DoudizhuUiState};
use crate::app::presentation::{CardCounterCell, TEXT, add_card_counter_window, add_switch_drawer};
use crate::app::runtime::UiAssets;
use crate::app::shell::UiAction;
use bevy::prelude::*;
use leocard_doudizhu::DoudizhuCard;
use leocard_protocol::DoudizhuSnapshot;
use leocard_ui::{DrawerSwitch, SwitchDrawer, UiComposer};
use std::array;
use std::collections::HashSet;

pub(super) fn render(
    commands: &mut UiComposer,
    parent: Entity,
    game: &DoudizhuSnapshot,
    ui: &DoudizhuUiState,
    assets: &UiAssets,
) {
    if !ui.counter.active {
        return;
    }
    add_switch_drawer(
        commands,
        parent,
        SwitchDrawer {
            expanded: ui.counter.drawer_open,
            toggle_action: UiAction::Doudizhu(DoudizhuUiAction::ToggleCounterDrawer),
            switches: &[DrawerSwitch {
                short: "牌",
                label: "记牌器",
                enabled: ui.counter.enabled,
                action: UiAction::Doudizhu(DoudizhuUiAction::ToggleCounter),
            }],
        },
        assets,
    );
    if !ui.counter.enabled {
        return;
    }
    let excluded: HashSet<_> = game.your_hand.iter().chain(&game.played).copied().collect();
    let rows = [3, 2, 1, 0].map(|suit| {
        array::from_fn(|column| {
            let card = if column == 0 {
                match suit {
                    3 => DoudizhuCard(53),
                    1 => DoudizhuCard(52),
                    _ => return None,
                }
            } else {
                DoudizhuCard((13 - column) as u8 * 4 + suit)
            };
            let known = excluded
                .iter()
                .filter(|c| c.rank() == card.rank() && (card.rank() >= 13 || c.suit() == suit))
                .count() as u8;
            Some(CardCounterCell {
                image: assets.playing_cards.face(card),
                remaining: game.rules.mode.deck_count().saturating_sub(known),
                color: TEXT,
            })
        })
    });
    add_card_counter_window::<DoudizhuUiState>(commands, parent, &ui.counter, rows, assets);
}
