use super::{QiGuiActionContext, QiGuiActionStatistics, QiGuiMatchStatistics};
use crate::{GameResult, GameState};

impl QiGuiMatchStatistics {
    pub(super) fn observe_finish(
        &mut self,
        before: &QiGuiActionContext,
        result: &GameResult,
        game: &GameState,
        reports: &mut [QiGuiActionStatistics],
    ) {
        self.finished = true;
        let highest = result.scores.iter().copied().max().unwrap_or(0);
        let winner = &mut reports[result.finisher.0];
        // End-of-game collection is separate from the final trick's cards.
        for player in game.players() {
            winner.capture(player.hand());
        }
        winner.captured_points += result.captured_hand_points;
        winner.comeback = before.scores[result.finisher.0]
            < before.scores.iter().copied().max().unwrap_or(0)
            && result.scores[result.finisher.0] == highest
            && result
                .scores
                .iter()
                .filter(|score| **score == highest)
                .count()
                == 1;
        for report in reports {
            let index = report.player.0;
            report.completed_game = true;
            report.won = result.scores[index] == highest;
            report.all_points = self.total_points > 0 && result.scores[index] == self.total_points;
            report.uncontested_win = report.won
                && self
                    .players
                    .iter()
                    .enumerate()
                    .all(|(other, p)| other == index || p.plays == 0);
        }
    }
}
