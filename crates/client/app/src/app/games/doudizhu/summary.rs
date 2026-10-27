use crate::app::presentation::{
    GameSummaryAnimation, PlayerMenuProfile, RankedSummary, SummaryDescriptor, SummaryPlayer,
    SummaryScore, add_ranked_summary,
};
use crate::app::runtime::{AvatarImages, UiAssets};
use crate::app::shell::{SummaryActions, add_summary_actions};
use bevy::prelude::*;
use leocard_protocol::DoudizhuSnapshot;
use leocard_ui::UiComposer;

pub(crate) fn descriptor(game: &DoudizhuSnapshot) -> Option<SummaryDescriptor> {
    let result = game.result.as_ref()?;
    Some(SummaryDescriptor {
        match_id: game.match_id,
        settlement_index: None,
        entry_count: game.players.len(),
        nonnegative_outcome: result
            .scores
            .get(usize::from(game.you.0))
            .is_none_or(|s| *s >= 0),
        reveal_duration: 0.4,
    })
}

pub(super) fn render(
    commands: &mut UiComposer,
    parent: Entity,
    game: &DoudizhuSnapshot,
    avatars: &AvatarImages,
    animation: &GameSummaryAnimation,
    assets: &UiAssets,
) {
    let Some(result) = &game.result else {
        return;
    };
    let landlord = game
        .players
        .iter()
        .find(|player| Some(player.id) == game.landlord)
        .map_or("玩家", |player| player.name.as_str());
    let actions = add_ranked_summary(
        commands,
        parent,
        &RankedSummary {
            players: game
                .players
                .iter()
                .map(|player| SummaryPlayer {
                    id: player.id,
                    profile: PlayerMenuProfile {
                        name: &player.name,
                        avatar: player.avatar.and_then(|id| avatars.remote.get(&id)),
                        reference_points: player.reference_points,
                        completed_games: player.completed_games,
                        game_profiles: &player.game_profiles,
                    },
                    ready: player.ready,
                })
                .collect(),
            scores: game
                .players
                .iter()
                .map(|player| {
                    SummaryScore::signed(player.id, result.scores[usize::from(player.id.0)])
                })
                .collect(),
            reference_changes: &[],
            title: if result.landlord_won {
                "地主获胜"
            } else {
                "农民获胜"
            },
            caption: &format!(
                "地主 {landlord}  底分 {}  倍数 ×{}{}",
                game.highest_bid,
                game.multiplier,
                if result.spring { "  春天" } else { "" }
            ),
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
                .find(|player| player.id == game.you)
                .map(|player| player.ready),
            again: "再来一局",
        },
        assets,
    );
}
