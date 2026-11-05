use crate::app::presentation::{
    GameSummaryAnimation, PlayerMenuProfile, RankedSummary, SummaryDescriptor, SummaryPlayer,
    SummaryRows, add_ranked_summary,
};
use crate::app::runtime::{AvatarImages, UiAssets};
use crate::app::shell::{
    CozyButtonVariant, LobbyUiAction, SummaryActions, UiAction, add_cozy_button_variant,
    add_cozy_disabled_button, add_summary_actions,
};
use bevy::prelude::*;
use leocard_guandan::rank_label;
use leocard_protocol::GuandanSnapshot;
use leocard_ui::UiComposer;

pub(crate) fn descriptor(game: &GuandanSnapshot) -> Option<SummaryDescriptor> {
    let result = game.result.as_ref()?;
    let own = game
        .players
        .iter()
        .find(|player| player.player.id == game.you)?;
    Some(SummaryDescriptor {
        match_id: game.match_id,
        settlement_index: Some(game.hand_number),
        entry_count: game.players.len(),
        nonnegative_outcome: own.seat.0 % 2 == result.winning_team,
        reveal_duration: 0.4,
    })
}

pub(super) fn render(
    commands: &mut UiComposer,
    parent: Entity,
    game: &GuandanSnapshot,
    avatars: &AvatarImages,
    animation: &GameSummaryAnimation,
    assets: &UiAssets,
) {
    let Some(result) = &game.result else {
        return;
    };
    let spectator_animation = GameSummaryAnimation {
        elapsed: 10.0,
        ..default()
    };
    let animation = if game
        .players
        .iter()
        .any(|player| player.player.id == game.you)
    {
        animation
    } else {
        &spectator_animation
    };
    let winning_team = if result.winning_team == 0 {
        "一队"
    } else {
        "二队"
    };
    let next_level = game.team_levels[usize::from(result.winning_team)];
    let caption = if game.match_complete {
        format!("第 {} 局  {} 双上过 A", game.hand_number, winning_team)
    } else if game.level_advance == 0 {
        format!(
            "第 {} 局  {} 获胜  下局仍打 {}",
            game.hand_number,
            winning_team,
            rank_label(next_level),
        )
    } else {
        format!(
            "第 {} 局  {} 升 {} 级  下局打 {}",
            game.hand_number,
            winning_team,
            game.level_advance,
            rank_label(next_level),
        )
    };
    let actions = add_ranked_summary(
        commands,
        parent,
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
            rows: SummaryRows::Placements(
                result
                    .finishing_order
                    .iter()
                    .enumerate()
                    .map(|(index, &player)| {
                        let label = (index < usize::from(result.known_positions))
                            .then(|| placement_label(index));
                        (player, label)
                    })
                    .collect(),
            ),
            reference_changes: &game.reference_changes,
            title: if game.match_complete {
                "整场结算"
            } else {
                "本局结算"
            },
            caption: &caption,
        },
        assets,
        animation,
    );
    let ready = game
        .players
        .iter()
        .find(|player| player.player.id == game.you)
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
        commands
            .entity(actions)
            .entry::<Node>()
            .and_modify(|mut node| node.justify_content = JustifyContent::Center);
        match ready {
            Some(false) => {
                add_cozy_button_variant(
                    commands,
                    actions,
                    "准备下一局",
                    UiAction::Lobby(LobbyUiAction::PlayAgain),
                    assets,
                    px(150),
                    48.0,
                    CozyButtonVariant::Cool,
                );
            }
            Some(true) => {
                add_cozy_disabled_button(commands, actions, "已准备", assets, px(150), 48.0);
            }
            None => {
                add_cozy_disabled_button(commands, actions, "观战中", assets, px(150), 48.0);
            }
        }
    }
}

pub(super) fn placement_label(index: usize) -> &'static str {
    ["头游", "二游", "三游", "末游"][index]
}
