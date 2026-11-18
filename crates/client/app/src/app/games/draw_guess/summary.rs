use super::screen::DrawGuessScreen;
use super::view::DrawingContext;
use crate::app::games::GameSummary;
use crate::app::presentation::{
    PlayerMenuProfile, RankedSummary, SummaryDescriptor, SummaryPlayer, SummaryRows,
    add_ranked_summary,
};
use crate::app::shell::{SummaryActions, add_summary_actions};
use bevy::prelude::*;
use leocard_protocol::{DrawGuessSnapshot, PlayerScore};
use leocard_ui::UiComposer;

pub(super) fn render(
    commands: &mut UiComposer,
    parent: Entity,
    game: &DrawGuessSnapshot,
    context: &DrawingContext<'_>,
) {
    if !game.match_complete {
        return;
    }
    let caption = format!(
        "共 {} 轮 {} 局",
        game.rules.rounds,
        game.players.len() * usize::from(game.rules.rounds)
    );
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
                        avatar: if player.player.id == game.you {
                            context.avatars.local.as_ref()
                        } else {
                            player
                                .player
                                .avatar
                                .and_then(|id| context.avatars.remote.get(&id))
                        },
                        reference_points: player.player.reference_points,
                        completed_games: player.player.completed_games,
                        game_profiles: &player.player.game_profiles,
                    },
                    ready: player.ready,
                })
                .collect(),
            rows: SummaryRows::Scores(
                game.state
                    .players
                    .iter()
                    .map(|player| {
                        PlayerScore {
                            player: player.id,
                            score: player.score,
                        }
                        .into()
                    })
                    .collect(),
                "分",
            ),
            reference_changes: &game.reference_changes,
            title: "整场结算",
            caption: &caption,
        },
        context.assets,
        context.summary,
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
            again: "开始下一场",
        },
        context.assets,
    );
}

impl GameSummary for DrawGuessScreen<'_> {
    fn summary_descriptor(&self, game: &Self::Snapshot) -> Option<SummaryDescriptor> {
        game.match_complete.then(|| SummaryDescriptor {
            match_id: game.match_id,
            settlement_index: None,
            entry_count: game.players.len(),
            reveal_duration: 1.5,
            nonnegative_outcome: game
                .reference_changes
                .iter()
                .find(|change| change.player == game.you)
                .is_none_or(|change| change.delta >= 0),
        })
    }

    fn summary_pause(&self, _game: &Self::Snapshot) -> Option<f32> {
        self.feedback.scores_busy().then_some(-0.01)
    }
}
