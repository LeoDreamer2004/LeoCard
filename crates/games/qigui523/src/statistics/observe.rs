use super::{
    QiGuiActionContext, QiGuiActionStatistics, QiGuiMatchStatistics, QiGuiPlayerStatistics,
    state::PlayerRuns,
};
use crate::{ActionOutcome, ClassifiedPlay, GameState, Phase, QiGuiCard, QiGuiPlayerId, QiGuiRank};

impl QiGuiMatchStatistics {
    pub fn new(game: &GameState) -> Self {
        let mut statistics = Self {
            players: vec![Default::default(); game.players().len()],
            runs: vec![PlayerRuns::default(); game.players().len()],
            completed_tricks: 0,
            total_points: game.total_points(),
            finished: false,
        };
        statistics.observe_hands(game);
        statistics
    }

    pub fn players(&self) -> &[QiGuiPlayerStatistics] {
        &self.players
    }

    pub fn observe(
        &mut self,
        before: QiGuiActionContext,
        outcome: &ActionOutcome,
        play: Option<&ClassifiedPlay>,
        game: &GameState,
    ) -> Vec<QiGuiActionStatistics> {
        if self.finished {
            return Vec::new();
        }
        let mut reports = self
            .players
            .iter()
            .enumerate()
            .map(|(index, _)| QiGuiActionStatistics::new(QiGuiPlayerId(index), *game.rules()))
            .collect::<Vec<_>>();
        for (progress, hand) in self.players.iter_mut().zip(&before.hands) {
            progress.observe_hand(hand, game.rules().hand_size);
        }
        self.observe_play(&before, play, &mut reports[before.actor.0]);
        self.observe_trick(&before, outcome, play, &mut reports);
        self.observe_hands(game);
        if let Phase::Finished(result) = game.phase() {
            self.observe_finish(&before, result, game, &mut reports);
        }
        for (report, progress) in reports.iter_mut().zip(&self.players) {
            report.progress = progress.clone();
        }
        reports
    }

    fn observe_hands(&mut self, game: &GameState) {
        for (statistics, player) in self.players.iter_mut().zip(game.players()) {
            statistics.observe_hand(player.hand(), game.rules().hand_size);
        }
    }
}

impl QiGuiPlayerStatistics {
    fn observe_hand(&mut self, hand: &[QiGuiCard], capacity: u8) {
        self.full_high_hand |= hand.len() == usize::from(capacity)
            && hand
                .iter()
                .all(|card| card.rank().strength() >= QiGuiRank::Three.strength());
    }
}
