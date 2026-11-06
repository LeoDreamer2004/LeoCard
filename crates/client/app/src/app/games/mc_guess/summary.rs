use super::screen::McGuessScreen;
use super::view::GuessTable;
use crate::app::games::GameSummary;
use crate::app::presentation::{
    PlayerMenuProfile, RankedSummary, SummaryDescriptor, SummaryPlayer, SummaryRows,
    add_ranked_summary,
};
use crate::app::shell::{SummaryActions, add_summary_actions};
use bevy::prelude::*;
use leocard_ui::UiComposer;

impl GuessTable<'_> {
    pub(super) fn summary(&self, commands: &mut UiComposer, page: Entity) {
        if !self.game.round.finished() || !self.game.progress.is_last_game() {
            return;
        }
        let progress = self.game.progress;
        let caption = match progress.round_count() {
            Some(rounds) => format!("共 {rounds} 轮 {} 局", progress.total_games()),
            None if self.game.graph.is_some() => {
                format!("关系谜图 共 {} 局", progress.total_games())
            }
            None => format!("抢答竞速 共 {} 局", progress.total_games()),
        };
        let actions = add_ranked_summary(
            commands,
            page,
            &RankedSummary {
                players: self
                    .game
                    .players
                    .iter()
                    .map(|player| SummaryPlayer {
                        id: player.player.id,
                        profile: PlayerMenuProfile {
                            name: &player.player.name,
                            avatar: player
                                .player
                                .avatar
                                .and_then(|id| self.avatars.remote.get(&id)),
                            reference_points: player.player.reference_points,
                            completed_games: player.player.completed_games,
                            game_profiles: &player.player.game_profiles,
                        },
                        ready: player.ready,
                    })
                    .collect(),
                rows: SummaryRows::Scores(
                    self.game.scores.iter().copied().map(Into::into).collect(),
                    "分",
                ),
                reference_changes: &self.game.reference_changes,
                title: "整场结算",
                caption: &caption,
            },
            self.assets,
            self.summary,
        );
        add_summary_actions(
            commands,
            actions,
            SummaryActions {
                is_host: self.game.you == self.game.host,
                ready: self
                    .game
                    .players
                    .iter()
                    .find(|player| player.player.id == self.game.you)
                    .map(|player| player.ready),
                again: "开始下一场",
            },
            self.assets,
        );
    }
}

impl GameSummary for McGuessScreen<'_> {
    fn summary_descriptor(&self, game: &Self::Snapshot) -> Option<SummaryDescriptor> {
        (game.round.finished() && game.progress.is_last_game()).then(|| SummaryDescriptor {
            match_id: game.match_id,
            settlement_index: None,
            entry_count: game.scores.len(),
            nonnegative_outcome: game
                .reference_changes
                .iter()
                .find(|change| change.player == game.you)
                .is_none_or(|change| change.delta >= 0),
            reveal_duration: 1.5,
        })
    }

    fn summary_pause(&self, _game: &Self::Snapshot) -> Option<f32> {
        (self.scores.busy() || self.feedback.busy()).then_some(-0.01)
    }
}
