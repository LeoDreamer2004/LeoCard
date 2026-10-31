use crate::app::presentation::{
    GameSummaryAnimation, PlayerMenuProfile, RankedSummary, SummaryDescriptor, SummaryPlayer,
    add_ranked_summary,
};
use crate::app::runtime::{AvatarImages, UiAssets};
use crate::app::shell::{SummaryActions, add_summary_actions};
use leocard_protocol::{QiGui523PhaseView, QiGui523Snapshot};
use leocard_ui::UiComposer;

use bevy::prelude::*;

pub(crate) fn qigui523_summary_descriptor(game: &QiGui523Snapshot) -> Option<SummaryDescriptor> {
    let QiGui523PhaseView::Finished {
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
        settlement_index: None,
        entry_count: scores.len(),
        nonnegative_outcome: reference_changes
            .iter()
            .find(|change| change.player == game.you)
            .is_none_or(|change| change.delta >= 0),
        reveal_duration: 1.5,
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
    let QiGui523PhaseView::Finished {
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
        .find(|player| player.player.id == *finisher)
        .map_or("玩家", |player| player.player.name.as_str());
    let caption = format!("{finisher_name} 率先出完手牌");
    let actions = add_ranked_summary(
        commands,
        table,
        &RankedSummary {
            players: game
                .players
                .iter()
                .map(|player| SummaryPlayer {
                    id: player.player.id,
                    profile: PlayerMenuProfile {
                        name: &player.player.name,
                        avatar: player.player.avatar.and_then(|id| avatars.remote.get(&id)),
                        reference_points: player.player.reference_points,
                        completed_games: player.player.completed_games,
                        game_profiles: &player.player.game_profiles,
                    },
                    ready: player.ready,
                })
                .collect(),
            scores: scores.iter().copied().map(Into::into).collect(),
            reference_changes,
            title: "本局结算",
            caption: &caption,
            score_unit: "分",
        },
        assets,
        animation,
    );
    add_summary_actions(
        commands,
        actions,
        SummaryActions {
            is_host: game.you == game.host,
            ready: game
                .players
                .iter()
                .find(|player| player.player.id == game.you)
                .map(|player| player.ready),
            again: "再来一局",
        },
        assets,
    );
}
