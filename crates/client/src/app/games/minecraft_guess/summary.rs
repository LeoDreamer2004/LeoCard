use super::super::summary::{RankedSummary, SummaryPlayer, add_ranked_summary};
use super::view::GuessTable;
use crate::app::presentation::{SUMMARY_HAND_REVEAL_DURATION, SummaryDescriptor, UiComposer};
use bevy::prelude::*;
use leocard_protocol::MinecraftGuessSnapshot;

pub(crate) fn guess_summary_descriptor(game: &MinecraftGuessSnapshot) -> Option<SummaryDescriptor> {
    (game.round.finished() && game.progress.is_last_game()).then(|| SummaryDescriptor {
        match_id: game.match_id,
        texas_hand_number: None,
        settlement_index: None,
        entry_count: game.scores.len(),
        nonnegative_outcome: game
            .reference_changes
            .iter()
            .find(|change| change.player == game.you)
            .is_none_or(|change| change.delta >= 0),
        reveal_duration: SUMMARY_HAND_REVEAL_DURATION,
    })
}

impl GuessTable<'_> {
    pub(super) fn summary(&self, commands: &mut UiComposer, page: Entity) {
        if guess_summary_descriptor(self.game).is_none() {
            return;
        }
        let caption = format!(
            "共 {} 轮 · {} 局",
            self.game.progress.rounds,
            self.game.progress.total_games()
        );
        add_ranked_summary(
            commands,
            page,
            &RankedSummary {
                you: self.game.you,
                host: self.game.host,
                players: self
                    .game
                    .players
                    .iter()
                    .map(|player| SummaryPlayer {
                        id: player.id,
                        name: &player.name,
                        avatar: player.avatar,
                        ready: player.ready,
                    })
                    .collect(),
                scores: &self.game.scores,
                reference_changes: &self.game.reference_changes,
                title: "整场结算",
                caption: &caption,
                again: "开始下一场",
            },
            self.assets,
            self.avatars,
            self.summary,
        );
    }
}
