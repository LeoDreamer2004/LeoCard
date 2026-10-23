use super::GraphGame;
use crate::{GuessActionStatistics, GuessHandStatistics, GuessPlayStatistics};

impl GraphGame {
    pub(crate) fn take_action_statistics(&mut self) -> Vec<GuessActionStatistics> {
        self.analysis.take_actions()
    }

    pub(crate) fn take_hand_statistics(&mut self) -> Vec<GuessHandStatistics> {
        self.analysis.take_hands()
    }

    pub(crate) fn statistics(&self) -> impl Iterator<Item = (u8, GuessPlayStatistics)> + '_ {
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
