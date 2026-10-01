use super::TexasHoldemActionStatistics;
use crate::{
    GameState, Phase, TexasHoldemAction, TexasHoldemPlayerId, TexasHoldemRuleSet, TexasHoldemStreet,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct ActionRecord {
    pub player: TexasHoldemPlayerId,
    pub action: TexasHoldemAction,
    pub statistics: TexasHoldemActionStatistics,
    pub facing_raise: bool,
    pub side_pot_raise: bool,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct HandHistory {
    pub(super) actions: Vec<ActionRecord>,
}

impl HandHistory {
    pub(crate) fn record(
        before: &GameState,
        after: &GameState,
        player: TexasHoldemPlayerId,
        action: TexasHoldemAction,
    ) -> Self {
        let mut history = before.statistics.history.clone();
        let old = &before.players()[player.0];
        let new = &after.players()[player.0];
        let amount = new.committed_total() - old.committed_total();
        let target = old.committed_street() + amount;
        let voluntary = action != TexasHoldemAction::PostBlind;
        let raised = voluntary && target > before.current_bet();
        let full_raise = raised && target >= before.minimum_raise_to();
        let Phase::Betting(street) = before.phase() else {
            unreachable!("accepted action needs a betting phase")
        };
        let earlier_raises = history
            .actions
            .iter()
            .filter(|record| record.statistics.street == *street && record.statistics.full_raise)
            .count() as u32;
        let bet_level = earlier_raises
            + u32::from(full_raise)
            + u32::from(*street == TexasHoldemStreet::PreFlop);
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
            statistics: TexasHoldemActionStatistics {
                street: *street,
                all_in_amount: if voluntary && new.all_in() { target } else { 0 },
                full_raise,
                raised,
                bet_level,
            },
            facing_raise: *street == TexasHoldemStreet::PreFlop
                && before.current_bet() > TexasHoldemRuleSet::BIG_BLIND
                && target >= before.current_bet()
                && amount > 0,
            side_pot_raise: raised && all_in_players >= 2 && actionable >= 2,
        });
        history
    }
}

impl GameState {
    pub fn last_action_statistics(&self) -> Option<TexasHoldemActionStatistics> {
        self.statistics
            .history
            .actions
            .last()
            .map(|record| record.statistics)
    }
}
