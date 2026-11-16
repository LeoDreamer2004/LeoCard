use super::state::GuandanUiState;
use crate::app::games::{CardCounter, CounterRows};
use crate::app::presentation::{
    ACCENT, CardCounterCell, CardCounterWindowState, TEXT, add_card_counter_window,
};
use crate::app::runtime::{ClientResource, UiAssets};
use crate::app::shell::UiState;
use bevy::prelude::*;
use leocard_client::{PlayerEconomy, PropId};
use leocard_guandan::{GuandanCard, GuandanPhase, GuandanSuit};
use leocard_protocol::{GameSnapshot, GuandanSnapshot};
use leocard_ui::UiComposer;
use std::array;
use std::collections::HashSet;

pub(super) fn sync_counter(
    client: Option<Res<ClientResource>>,
    economy: Res<PlayerEconomy>,
    mut game_ui: ResMut<GuandanUiState>,
    mut ui: ResMut<UiState>,
) {
    let game = client.as_deref().and_then(|client| {
        client
            .0
            .model()
            .game_snapshot()
            .and_then(GameSnapshot::guandan)
    });
    let active = game.is_some_and(|game| matches!(game.phase, GuandanPhase::Playing))
        && economy.active(PropId::CardCounter);
    if game_ui.counter.active != active {
        game_ui.counter.active = active;
        game_ui.counter.drag.grab = None;
        ui.dirty = true;
    }
}

pub(super) fn render(
    commands: &mut UiComposer,
    viewport: Entity,
    game: &GuandanSnapshot,
    ui: &GuandanUiState,
    assets: &UiAssets,
) {
    if !ui.counter_window().active {
        return;
    }
    if !ui.counter.enabled {
        return;
    }
    add_card_counter_window::<GuandanUiState>(
        commands,
        viewport,
        ui.counter_window(),
        ui.counter_rows(game, assets),
        assets,
    );
}

impl CardCounter for GuandanUiState {
    type Context = GuandanSnapshot;

    fn counter_window(&self) -> &CardCounterWindowState {
        &self.counter
    }

    fn counter_rows(&self, game: &Self::Context, assets: &UiAssets) -> CounterRows {
        let known: HashSet<_> = game
            .your_hand
            .iter()
            .chain(&game.played_cards)
            .copied()
            .collect();
        [
            GuandanSuit::Spades,
            GuandanSuit::Hearts,
            GuandanSuit::Clubs,
            GuandanSuit::Diamonds,
        ]
        .map(|suit| {
            array::from_fn(|column| {
                let card = if column == 0 {
                    match suit {
                        GuandanSuit::Spades => GuandanCard {
                            deck: 0,
                            suit: GuandanSuit::Joker,
                            rank: 17,
                        },
                        GuandanSuit::Hearts => GuandanCard {
                            deck: 0,
                            suit: GuandanSuit::Joker,
                            rank: 16,
                        },
                        _ => return None,
                    }
                } else {
                    GuandanCard {
                        deck: 0,
                        suit,
                        rank: 15 - column as u8,
                    }
                };
                let count = known
                    .iter()
                    .filter(|known| known.suit == card.suit && known.rank == card.rank)
                    .count() as u8;
                Some(CardCounterCell {
                    image: assets.playing_cards.face(card),
                    remaining: 2_u8.saturating_sub(count),
                    color: if card.rank == game.level {
                        ACCENT
                    } else {
                        TEXT
                    },
                })
            })
        })
    }
}
