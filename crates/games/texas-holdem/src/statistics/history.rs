use super::TexasActionStatistics;
use crate::{TexasAction, TexasGameState, TexasPhase, TexasRuleSet, TexasStreet};
use leocard_game_common::PlayerId;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct ActionRecord {
    pub player: PlayerId,
    pub action: TexasAction,
    pub statistics: TexasActionStatistics,
    pub facing_raise: bool,
    pub side_pot_raise: bool,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct HandHistory {
    pub(super) actions: Vec<ActionRecord>,
}

impl HandHistory {
    pub(crate) fn record(
        before: &TexasGameState,
        after: &TexasGameState,
        player: PlayerId,
        action: TexasAction,
    ) -> Self {
        let mut history = before.statistics.history.clone();
        let old = &before.players()[player.index()];
        let new = &after.players()[player.index()];
        let amount = new.committed_total() - old.committed_total();
        let target = old.committed_street() + amount;
        let voluntary = action != TexasAction::PostBlind;
        let raised = voluntary && target > before.current_bet();
        let full_raise = raised && target >= before.minimum_raise_to();
        let TexasPhase::Betting(street) = before.phase() else {
            unreachable!("accepted action needs a betting phase")
        };
        let earlier_raises = history
            .actions
            .iter()
            .filter(|record| record.statistics.street == *street && record.statistics.full_raise)
            .count() as u32;
        let bet_level =
            earlier_raises + u32::from(full_raise) + u32::from(*street == TexasStreet::PreFlop);
        let all_in_players = before
            .players()
            .iter()
            .filter(|other| other.all_in() && !other.folded())
            .count();
        let actionable = before
            .players()
            .iter()
            .filter(|other| !other.all_in() && !other.folded())
            .count();
        history.actions.push(ActionRecord {
            player,
            action,
            statistics: TexasActionStatistics {
                street: *street,
                all_in_amount: if voluntary && new.all_in() { target } else { 0 },
                full_raise,
                raised,
                bet_level,
            },
            facing_raise: *street == TexasStreet::PreFlop
                && before.current_bet() > TexasRuleSet::BIG_BLIND
                && target >= before.current_bet()
                && amount > 0,
            side_pot_raise: raised && all_in_players >= 2 && actionable >= 2,
        });
        history
    }
}

impl TexasGameState {
    pub fn last_action_statistics(&self) -> Option<TexasActionStatistics> {
        self.statistics
            .history
            .actions
            .last()
            .map(|record| record.statistics)
    }
}
