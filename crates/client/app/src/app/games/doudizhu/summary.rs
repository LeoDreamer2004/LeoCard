use crate::app::presentation::{
    GameButtonSpec, GameButtonTone, GameSummaryAnimation, PlayerMenuProfile, RankedSummary,
    SummaryDescriptor, SummaryPlayer, SummaryScore, add_ranked_summary, add_textured_game_button,
};
use crate::app::runtime::{AvatarImages, UiAssets};
use crate::app::shell::{LobbyUiAction, SummaryActions, UiAction, add_summary_actions};
use bevy::prelude::*;
use leocard_protocol::DoudizhuSnapshot;
use leocard_ui::UiComposer;

pub(crate) fn descriptor(game: &DoudizhuSnapshot) -> Option<SummaryDescriptor> {
    let result = game.result.as_ref()?;
    Some(SummaryDescriptor {
        match_id: game.match_id,
        settlement_index: Some(u32::from(game.progress.hand_number)),
        entry_count: game.players.len(),
        nonnegative_outcome: if game.match_complete {
            game.scores
                .get(usize::from(game.you.0))
                .is_none_or(|score| *score >= 0)
        } else {
            result
                .scores
                .get(usize::from(game.you.0))
                .is_none_or(|score| *score >= 0)
        },
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
    let caption = if game.match_complete {
        format!(
            "{} 轮 共 {} 局 按累计得分排名",
            game.rules.rounds,
            game.progress.total_hands()
        )
    } else {
        format!(
            "第 {}/{} 轮 第 {}/{} 局 地主 {landlord} 底分 {} 倍数 ×{}{}",
            game.progress.round(),
            game.rules.rounds,
            game.progress.hand_in_round(),
            game.rules.mode.players(),
            game.highest_bid,
            game.multiplier,
            if result.spring { " 春天" } else { "" },
        )
    };
    let scores = if game.match_complete {
        &game.scores
    } else {
        &result.scores
    };
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
                .map(|player| SummaryScore::signed(player.id, scores[usize::from(player.id.0)]))
                .collect(),
            reference_changes: &game.reference_changes,
            title: if game.match_complete {
                "整场结算"
            } else if result.landlord_won {
                "地主获胜"
            } else {
                "农民获胜"
            },
            caption: &caption,
            score_unit: "分",
        },
        assets,
        animation,
    );
    let ready = game
        .players
        .iter()
        .find(|player| player.id == game.you)
        .map(|player| player.ready);
    if game.match_complete {
        add_summary_actions(
            commands,
            actions,
            SummaryActions {
                is_host: game.you == game.host,
                ready,
                again: "开始下一场",
            },
            assets,
        );
    } else {
        add_textured_game_button(
            commands,
            actions,
            assets,
            GameButtonSpec::compact(
                assets,
                match ready {
                    Some(false) => "准备下一局",
                    Some(true) => "已准备",
                    None => "观战中",
                },
                (ready == Some(false)).then_some(UiAction::Lobby(LobbyUiAction::PlayAgain)),
                GameButtonTone::Play,
            ),
        );
    }
}
