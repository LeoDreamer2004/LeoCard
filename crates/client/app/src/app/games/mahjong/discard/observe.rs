use super::super::MahjongUiState;
use super::{MahjongOwnDiscardAnimation, MahjongRemoteDiscardAnimation};
use crate::app::presentation::mahjong::REFERENCE_DESIGN_HEIGHT;
use bevy::prelude::*;
use leocard_protocol::MahjongSnapshot;

pub(in super::super) fn observe_discard_animations(
    ui: &mut MahjongUiState,
    game: &MahjongSnapshot,
) {
    let previous = &ui.observed_table.state;
    if let Some(previous_count) = previous.discard_count
        && game.discards.len() == previous_count + 1
        && previous.hand.len() == game.your_hand.len() + 1
        && let Some(discard) = game.discards.last()
        && discard.player == game.you
        && discard.claimed_by.is_none()
        && previous.hand.contains(&discard.tile)
        && !game.your_hand.contains(&discard.tile)
    {
        let river_slot = game
            .discards
            .iter()
            .filter(|entry| entry.player == game.you && entry.claimed_by.is_none())
            .count()
            - 1;
        ui.discard_animation = Some(MahjongOwnDiscardAnimation {
            tile: discard.tile,
            old_hand: previous.hand.clone(),
            old_drawn: previous.drawn_tile,
            old_meld_count: previous.meld_count,
            discard_index: game.discards.len() - 1,
            river_slot,
            elapsed: 0.0,
            table_height: ui.last_table_height,
            table_viewport: Vec2::new(1280.0, REFERENCE_DESIGN_HEIGHT),
        });
    } else if ui.discard_animation.as_ref().is_some_and(|active| {
        game.discards
            .get(active.discard_index)
            .is_none_or(|discard| discard.claimed_by.is_some())
    }) {
        ui.discard_animation = None;
    }
    if let Some(previous_count) = previous.discard_count
        && game.discards.len() == previous_count + 1
        && let Some(discard) = game.discards.last()
        && discard.player != game.you
        && discard.claimed_by.is_none()
        && game
            .players
            .iter()
            .any(|player| player.player.id == discard.player)
    {
        ui.remote_discard_animation = Some(MahjongRemoteDiscardAnimation {
            player: discard.player,
            tile: discard.tile,
            discard_index: game.discards.len() - 1,
            from_drawn: discard.from_drawn,
            elapsed: 0.0,
            pose: None,
        });
    } else if ui.remote_discard_animation.as_ref().is_some_and(|active| {
        game.discards
            .get(active.discard_index)
            .is_none_or(|discard| discard.claimed_by.is_some())
    }) {
        ui.remote_discard_animation = None;
    }
}
