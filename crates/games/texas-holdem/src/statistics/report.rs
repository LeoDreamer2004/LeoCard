use super::TexasHoldemHandStatistics;
use super::pots::PotStatistics;
use crate::{
    GameState, Phase, TexasHoldemAction, TexasHoldemPlayerId, TexasHoldemStreet,
    evaluate_player_hand,
};

impl GameState {
    /// Only available after settlement; the caller must deliver this solely to `player`.
    pub fn hand_statistics(
        &self,
        player: TexasHoldemPlayerId,
    ) -> Option<TexasHoldemHandStatistics> {
        let Phase::Complete(result) = self.phase() else {
            return None;
        };
        let state = self.players().get(player.0)?;
        let own_actions = self
            .statistics
            .history
            .actions
            .iter()
            .filter(|record| record.player == player)
            .collect::<Vec<_>>();
        let evaluated = self
            .players()
            .iter()
            .map(|other| {
                evaluate_player_hand(other.hole_cards(), self.community(), self.rules()).ok()
            })
            .collect::<Vec<_>>();
        let category = evaluated[player.0].map(|hand| hand.category());
        let mut report = TexasHoldemHandStatistics {
            hole_cards: state.hole_cards().to_vec(),
            category,
            showdown: result.showdown,
            short_deck: self.rules().short_deck,
            button: player == self.dealer(),
            big_blind: player == self.big_blind(),
            under_the_gun: self
                .players()
                .iter()
                .filter(|other| !other.hole_cards().is_empty())
                .count()
                >= 3
                && (1..self.players().len())
                    .map(|distance| (self.big_blind().0 + distance) % self.players().len())
                    .find(|&index| !self.players()[index].hole_cards().is_empty())
                    == Some(player.0),
            defended_blind: player == self.big_blind()
                && own_actions.iter().any(|record| record.facing_raise),
            preflop_bet_levels: own_actions
                .iter()
                .filter(|record| {
                    record.statistics.street == TexasHoldemStreet::PreFlop
                        && record.statistics.full_raise
                })
                .map(|record| record.statistics.bet_level)
                .collect(),
            river_bet_called: self.river_bet_called(player),
            side_pot_raises: own_actions
                .iter()
                .filter(|record| record.side_pot_raise)
                .count() as u32,
            preflop_all_in: own_actions.iter().any(|record| {
                record.statistics.street == TexasHoldemStreet::PreFlop
                    && record.statistics.all_in_amount > 0
            }),
            final_stack: state.stack(),
            total_chips: result.final_stacks.iter().sum(),
            match_complete: result.final_stacks.contains(&0),
            first_place: result
                .final_stacks
                .iter()
                .all(|stack| *stack <= state.stack()),
            match_statistics: self.statistics.players[player.0].clone(),
            ..Default::default()
        };
        let pots = PotStatistics::new(self, result, player, &evaluated);
        report.won_chips = pots.won_chips;
        report.won_pot = pots.won_pot;
        report.won_side_pot_chips = pots.won_side_pot_chips;
        for index in pots.defeated {
            let other = &self.players()[index];
            report
                .defeated_categories
                .push(evaluated[index].unwrap().category());
            if let [first, second] = other.hole_cards()
                && first.rank() == second.rank()
            {
                report.defeated_pocket_pairs += 1;
                if self.statistics.history.actions.iter().any(|record| {
                    record.player.0 == index
                        && record.statistics.street == TexasHoldemStreet::PreFlop
                        && record.statistics.all_in_amount > 0
                }) {
                    report.defeated_preflop_all_in_pairs += 1;
                }
            }
        }
        report
            .defeated_categories
            .sort_by_key(|category| *category as u8);
        if let Some(index) = self.statistics.history.actions.iter().rposition(|record| {
            record.player == player
                && record.statistics.street == TexasHoldemStreet::River
                && record.statistics.raised
                && record.statistics.all_in_amount > 0
        }) {
            let subsequent = &self.statistics.history.actions[index + 1..];
            if !subsequent.iter().any(|record| record.statistics.raised) {
                report.river_all_in_folded_categories = subsequent
                    .iter()
                    .filter(|record| {
                        record.action == TexasHoldemAction::Fold
                            && record.statistics.street == TexasHoldemStreet::River
                    })
                    .filter_map(|record| evaluated[record.player.0].map(|hand| hand.category()))
                    .collect();
            }
        }
        Some(report)
    }

    fn river_bet_called(&self, player: TexasHoldemPlayerId) -> bool {
        self.statistics
            .history
            .actions
            .iter()
            .enumerate()
            .any(|(index, record)| {
                record.player == player
                    && record.statistics.street == TexasHoldemStreet::River
                    && record.statistics.raised
                    && self.statistics.history.actions[index + 1..]
                        .iter()
                        .take_while(|later| !later.statistics.raised)
                        .any(|later| {
                            later.player != player
                                && later.statistics.street == TexasHoldemStreet::River
                                && matches!(
                                    later.action,
                                    TexasHoldemAction::Call | TexasHoldemAction::AllIn
                                )
                        })
            })
    }
}
