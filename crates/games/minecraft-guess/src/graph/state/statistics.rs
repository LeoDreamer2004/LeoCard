use super::GraphGame;
use crate::GuessPlayStatistics;

impl GraphGame {
    pub(crate) fn statistics(&self) -> impl Iterator<Item = (u8, GuessPlayStatistics)> + '_ {
        assert!(self.finished());
        self.players
            .iter()
            .copied()
            .zip(&self.statistics)
            .map(|(player, statistics)| {
                let mut statistics = *statistics;
                for index in 0..self.nodes.len() {
                    let Some(entry) = self.entry(index) else {
                        continue;
                    };
                    let won = self.guesses.iter().any(|record| {
                        usize::from(record.node) == index + 1
                            && record.guess.player == player
                            && record.guess.result.is_correct()
                    });
                    statistics.finish_game(
                        entry.kind,
                        won,
                        self.rules.options.allow_unknown_guesses,
                    );
                }
                (player, statistics)
            })
    }
}
