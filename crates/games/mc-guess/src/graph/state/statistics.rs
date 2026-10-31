use super::McGuessGraphGame;
use crate::{McGuessActionStatistics, McGuessHandStatistics, McGuessPlayStatistics};
use leocard_game_common::PlayerId;

impl McGuessGraphGame {
    pub(crate) fn take_action_statistics(&mut self) -> Vec<McGuessActionStatistics> {
        self.analysis.take_actions()
    }

    pub(crate) fn take_hand_statistics(&mut self) -> Vec<McGuessHandStatistics> {
        self.analysis.take_hands()
    }

    pub(crate) fn statistics(
        &self,
    ) -> impl Iterator<Item = (PlayerId, McGuessPlayStatistics)> + '_ {
        assert!(self.finished());
        self.analysis
            .statistics(
                (0..self.nodes.len())
                    .filter_map(|index| self.entry(index).map(|entry| (index, entry.kind))),
                self.rules.options.allow_unknown_guesses,
            )
            .into_iter()
    }
}
