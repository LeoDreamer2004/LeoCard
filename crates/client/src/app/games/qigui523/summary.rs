use super::super::summary::{RankedSummary, SummaryPlayer, add_ranked_summary};
use crate::app::presentation::{
    GameSummaryAnimation, SUMMARY_HAND_REVEAL_DURATION, SummaryDescriptor, UiComposer,
};
use crate::app::runtime::{AvatarImages, UiAssets};
use bevy::prelude::*;
use leocard_protocol::{GamePhaseView, QiGui523Snapshot};

pub(crate) fn qigui523_summary_descriptor(game: &QiGui523Snapshot) -> Option<SummaryDescriptor> {
    let GamePhaseView::Finished {
        match_id,
        scores,
        reference_changes,
        ..
    } = &game.phase
    else {
        return None;
    };
    Some(SummaryDescriptor {
        match_id: *match_id,
        texas_hand_number: None,
        settlement_index: None,
        entry_count: scores.len(),
        nonnegative_outcome: reference_changes
            .iter()
            .find(|change| change.player == game.you)
            .is_none_or(|change| change.delta >= 0),
        reveal_duration: SUMMARY_HAND_REVEAL_DURATION,
    })
}

pub(super) fn add_game_summary_modal(
    commands: &mut UiComposer,
    table: Entity,
    game: &QiGui523Snapshot,
    assets: &UiAssets,
    avatars: &AvatarImages,
    animation: &GameSummaryAnimation,
) {
    let GamePhaseView::Finished {
        finisher,
        scores,
        reference_changes,
        ..
    } = &game.phase
    else {
        return;
    };
    let finisher_name = game
        .players
        .iter()
        .find(|player| player.id == *finisher)
        .map_or("玩家", |player| player.name.as_str());
    let caption = format!("{finisher_name} 率先出完手牌");
    add_ranked_summary(
        commands,
        table,
        &RankedSummary {
            you: game.you,
            host: game.host,
            players: game
                .players
                .iter()
                .map(|player| SummaryPlayer {
                    id: player.id,
                    name: &player.name,
                    avatar: player.avatar,
                    ready: player.ready,
                })
                .collect(),
            scores,
            reference_changes,
            title: "本局结算",
            caption: &caption,
            again: "再来一局",
        },
        assets,
        avatars,
        animation,
    );
}
