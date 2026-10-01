use super::history::{ActionRecord, HandHistory};
use super::{TexasHoldemHandStatistics, TexasHoldemMatchStatistics};
use crate::{GameState, Phase, PlayerState, TexasHoldemAction, TexasHoldemPlayerId};

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct GameStatistics {
    pub(super) history: HandHistory,
    pub(super) players: Vec<TexasHoldemMatchStatistics>,
}

impl GameStatistics {
    pub(crate) fn new(players: usize) -> Self {
        Self {
            history: HandHistory::default(),
            players: vec![TexasHoldemMatchStatistics::default(); players],
        }
    }

    pub(crate) fn start_hand(&mut self, players: &[PlayerState]) {
        self.history = HandHistory::default();
        for (player, statistics) in players.iter().zip(&mut self.players) {
            statistics.lowest_starting_stack = if statistics.hands == 0 {
                player.stack()
            } else {
                statistics.lowest_starting_stack.min(player.stack())
            };
        }
    }
}

impl GameState {
    pub(crate) fn record_action_statistics(
        &mut self,
        before: &Self,
        player: TexasHoldemPlayerId,
        action: TexasHoldemAction,
    ) {
        self.statistics.history = HandHistory::record(before, self, player, action);
        if matches!(self.phase(), Phase::Complete(_)) {
            self.record_hand_statistics();
        }
    }

    fn record_hand_statistics(&mut self) {
        let reports = (0..self.players().len())
            .map(|index| self.hand_statistics(TexasHoldemPlayerId(index)).unwrap())
            .collect::<Vec<_>>();
        let folded = self
            .players()
            .iter()
            .map(|player| player.folded())
            .collect::<Vec<_>>();
        for (index, (statistics, report)) in
            self.statistics.players.iter_mut().zip(reports).enumerate()
        {
            if report.hole_cards.is_empty() {
                continue;
            }
            statistics.record(
                &report,
                folded[index],
                &self.statistics.history.actions,
                TexasHoldemPlayerId(index),
            );
        }
    }
}

impl TexasHoldemMatchStatistics {
    fn record(
        &mut self,
        report: &TexasHoldemHandStatistics,
        folded: bool,
        actions: &[ActionRecord],
        player: TexasHoldemPlayerId,
    ) {
        self.hands = self.hands.saturating_add(1);
        self.won_hands = self
            .won_hands
            .saturating_add(u32::from(report.won_chips > 0));
        self.raised_hands = self.raised_hands.saturating_add(u32::from(
            actions
                .iter()
                .any(|record| record.player == player && record.statistics.raised),
        ));
        self.consecutive_folds = if folded {
            self.consecutive_folds.saturating_add(1)
        } else {
            0
        };
        if report.won_chips > 0
            && let Some(category) = report.category
        {
            if let Some(index) = self
                .consecutive_winning_categories
                .iter()
                .position(|previous| *previous == category)
            {
                self.consecutive_winning_categories.drain(..=index);
            }
            self.consecutive_winning_categories.push(category);
        } else {
            self.consecutive_winning_categories.clear();
        }
    }
}
