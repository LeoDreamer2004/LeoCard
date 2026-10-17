use super::BlackjackTable;
use crate::app::presentation::{
    RankedSummary, SummaryDescriptor, SummaryPlayer, UiComposer, add_ranked_summary,
};
use crate::app::shell::{SummaryActions, add_summary_actions};
use bevy::prelude::*;
use leocard_blackjack::Phase;
use leocard_protocol::{BlackjackSnapshot, PlayerScore};

pub(crate) fn blackjack_summary_descriptor(game: &BlackjackSnapshot) -> Option<SummaryDescriptor> {
    (game.phase == Phase::Complete).then(|| SummaryDescriptor {
        match_id: game.match_id,
        settlement_index: Some(u32::from(game.round)),
        entry_count: game.players.len(),
        nonnegative_outcome: game
            .players
            .iter()
            .find(|player| player.player.id == game.you)
            .is_none_or(|player| player.stack >= player.round_start_stack),
        reveal_duration: 1.2,
    })
}

impl BlackjackTable<'_> {
    pub(super) fn summary(&self, commands: &mut UiComposer, table: Entity) {
        if !self.game.match_complete || self.playback.busy() {
            return;
        }
        let scores: Vec<_> = self
            .game
            .players
            .iter()
            .map(|player| PlayerScore {
                player: player.player.id,
                score: player.stack,
            })
            .collect();
        let actions = add_ranked_summary(
            commands,
            table,
            &RankedSummary {
                players: self
                    .game
                    .players
                    .iter()
                    .map(|view| SummaryPlayer {
                        id: view.player.id,
                        name: &view.player.name,
                        avatar: view.player.avatar,
                        ready: view.player.ready,
                    })
                    .collect(),
                scores: &scores,
                reference_changes: &self.game.reference_changes,
                title: "整场结算",
                caption: &format!("21 点 · 共 {} 局", self.game.round),
                score_unit: "筹码",
            },
            self.assets,
            self.avatars,
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
                    .find(|view| view.player.id == self.game.you)
                    .is_some_and(|view| view.player.ready),
                again: "开始下一场",
            },
            self.assets,
        );
    }
}
