//! 双升高频快照的轻量差异判断。

use super::state::ShengjiUiState;
use crate::app::runtime::ClientResource;
use bevy::prelude::*;
use leocard_protocol::{MatchId, ShengjiFiveTrumpCrossingStage, ShengjiPhaseView, ShengjiSnapshot};

#[derive(Resource, Default)]
pub(super) struct ShengjiSelectionSync {
    observed_stage: Option<(MatchId, ShengjiFiveTrumpCrossingStage)>,
}

pub(super) fn sync_shengji_phase_selection(
    client: Option<Res<ClientResource>>,
    mut ui: ResMut<ShengjiUiState>,
    mut sync: ResMut<ShengjiSelectionSync>,
) {
    let game = client
        .as_deref()
        .and_then(|client| client.0.model().shengji_game());
    let current_stage = game.and_then(|game| match &game.phase {
        ShengjiPhaseView::FiveTrumpCrossing { stage, .. } => Some((game.match_id, *stage)),
        _ => None,
    });
    if current_stage == sync.observed_stage {
        return;
    }
    sync.observed_stage = current_stage;
    ui.selected.clear();

    let Some(game) = game else {
        return;
    };
    let ShengjiPhaseView::FiveTrumpCrossing {
        stage: ShengjiFiveTrumpCrossingStage::Deciding,
        eligible,
        decided,
        ..
    } = &game.phase
    else {
        return;
    };
    if !eligible.contains(&game.you) || decided.contains(&game.you) {
        return;
    }
    let Some(trump) = game.trump else {
        return;
    };
    ui.selected.extend(
        game.your_hand
            .iter()
            .copied()
            .filter(|card| trump.is_trump(*card)),
    );
}

pub(crate) fn only_shengji_transient_progress_changed(
    before: Option<&ShengjiSnapshot>,
    after: Option<&ShengjiSnapshot>,
) -> bool {
    let (Some(before), Some(after)) = (before, after) else {
        return false;
    };
    match (&before.phase, &after.phase) {
        (ShengjiPhaseView::Dealing { .. }, ShengjiPhaseView::Dealing { .. }) => {
            if before.your_hand != after.your_hand {
                return false;
            }
            let mut normalized = before.clone();
            normalized.phase = after.phase.clone();
            for player in &mut normalized.players {
                if let Some(latest) = after.players.iter().find(|latest| latest.id == player.id) {
                    player.hand_len = latest.hand_len;
                }
            }
            normalized == *after
        }
        (ShengjiPhaseView::BiddingGrace { .. }, ShengjiPhaseView::BiddingGrace { .. }) => {
            let mut normalized = before.clone();
            let ShengjiPhaseView::BiddingGrace {
                milliseconds_remaining,
                ..
            } = &mut normalized.phase
            else {
                unreachable!();
            };
            let ShengjiPhaseView::BiddingGrace {
                milliseconds_remaining: latest,
                ..
            } = &after.phase
            else {
                unreachable!();
            };
            *milliseconds_remaining = *latest;
            normalized == *after
        }
        (ShengjiPhaseView::BottomCopying { .. }, ShengjiPhaseView::BottomCopying { .. }) => {
            let mut normalized = before.clone();
            let ShengjiPhaseView::BottomCopying {
                milliseconds_remaining,
                ..
            } = &mut normalized.phase
            else {
                unreachable!();
            };
            let ShengjiPhaseView::BottomCopying {
                milliseconds_remaining: latest,
                ..
            } = &after.phase
            else {
                unreachable!();
            };
            *milliseconds_remaining = *latest;
            normalized == *after
        }
        _ => false,
    }
}
