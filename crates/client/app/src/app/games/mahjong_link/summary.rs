use crate::app::games::GameTableContext;
use crate::app::presentation::{
    PlayerMenuProfile, RankedSummary, SummaryDescriptor, SummaryPlayer, SummaryRows,
    add_ranked_summary,
};
use crate::app::shell::{SummaryActions, add_summary_actions};
use bevy::prelude::*;
use leocard_protocol::{MahjongLinkSnapshot, PlayerScore};
use leocard_ui::UiComposer;

pub(crate) fn descriptor(game: &MahjongLinkSnapshot) -> Option<SummaryDescriptor> {
    game.match_complete.then(|| SummaryDescriptor {
        match_id: game.match_id,
        settlement_index: None,
        entry_count: game.players.len(),
        nonnegative_outcome: game
            .players
            .iter()
            .find(|player| player.player.id == game.you)
            .is_none_or(|own| game.players.iter().all(|player| player.score <= own.score)),
        reveal_duration: 1.2,
    })
}

pub(super) fn render(
    commands: &mut UiComposer,
    table: Entity,
    game: &MahjongLinkSnapshot,
    context: &GameTableContext<'_>,
) {
    if !game.match_complete {
        return;
    }
    let actions = add_ranked_summary(
        commands,
        table,
        &RankedSummary {
            players: game
                .players
                .iter()
                .map(|view| SummaryPlayer {
                    id: view.player.id,
                    profile: PlayerMenuProfile {
                        name: &view.player.name,
                        avatar: if view.player.id == game.you {
                            context.avatars.local.as_ref()
                        } else {
                            view.player
                                .avatar
                                .and_then(|id| context.avatars.remote.get(&id))
                        },
                        reference_points: view.player.reference_points,
                        completed_games: view.player.completed_games,
                        game_profiles: &view.player.game_profiles,
                    },
                    ready: view.ready,
                })
                .collect(),
            rows: SummaryRows::Scores(
                game.players
                    .iter()
                    .map(|view| {
                        PlayerScore {
                            player: view.player.id,
                            score: u32::from(view.score),
                        }
                        .into()
                    })
                    .collect(),
                "分",
            ),
            reference_changes: &game.reference_changes,
            title: "本局结算",
            caption: if game.board.remaining() == 0 {
                "全部麻将已消除"
            } else {
                "全员连续跳过 按当前分数排名"
            },
        },
        context.assets,
        context.game_summary,
    );
    add_summary_actions(
        commands,
        actions,
        SummaryActions {
            is_host: game.you == game.host,
            ready: game
                .players
                .iter()
                .find(|view| view.player.id == game.you)
                .map(|view| view.ready),
            again: "再来一局",
        },
        context.assets,
    );
}
