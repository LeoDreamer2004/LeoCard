use super::TexasHandStatistics;
use super::pots::PotStatistics;
use crate::{TexasAction, TexasGameState, TexasPhase, TexasStreet, evaluate_player_hand};
use leocard_game_common::PlayerId;

impl TexasGameState {
    /// Only available after settlement; the caller must deliver this solely to `player`.
    pub fn hand_statistics(&self, player: PlayerId) -> Option<TexasHandStatistics> {
        let TexasPhase::Complete(result) = self.phase() else {
            return None;
        };
        let state = self.players().get(player.index())?;
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
                let first =
                    evaluate_player_hand(other.hole_cards(), self.community(), self.rules()).ok();
                let second = result.second_run.as_ref().and_then(|run| {
                    evaluate_player_hand(other.hole_cards(), &run.community, self.rules()).ok()
                });
                // A hand-level category is the strongest category across the runouts.
                first
                    .into_iter()
                    .chain(second)
                    .max_by(|left, right| left.cmp_with_rules(right, self.rules()))
            })
            .collect::<Vec<_>>();
        let category = if let Some(second) = &result.second_run {
            // Category-win achievements use the strongest board actually won, never a losing run.
            [
                (&result.community, &result.awards),
                (&second.community, &second.awards),
            ]
            .into_iter()
            .filter(|(_, awards)| {
                awards
                    .iter()
                    .any(|award| award.amount > 0 && award.winners.contains(&player))
            })
            .filter_map(|(community, _)| {
                evaluate_player_hand(state.hole_cards(), community, self.rules()).ok()
            })
            .max_by(|left, right| left.cmp_with_rules(right, self.rules()))
            .or(evaluated[player.index()])
            .map(|hand| hand.category())
        } else {
            evaluated[player.index()].map(|hand| hand.category())
        };
        let mut report = TexasHandStatistics {
            hole_cards: state.hole_cards().to_vec(),
            category,
            showdown: result.showdown,
            entered_pot: !state.hole_cards().is_empty()
                && !own_actions.iter().any(|record| {
                    record.action == TexasAction::Fold
                        && record.statistics.street == TexasStreet::PreFlop
                }),
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
                    .map(|distance| (self.big_blind().index() + distance) % self.players().len())
                    .find(|&index| !self.players()[index].hole_cards().is_empty())
                    == Some(player.index()),
            defended_blind: player == self.big_blind()
                && own_actions.iter().any(|record| record.facing_raise),
            preflop_bet_levels: own_actions
                .iter()
                .filter(|record| {
                    record.statistics.street == TexasStreet::PreFlop && record.statistics.full_raise
                })
                .map(|record| record.statistics.bet_level)
                .collect(),
            river_bet_called: self.river_bet_called(player),
            side_pot_raises: own_actions
                .iter()
                .filter(|record| record.side_pot_raise)
                .count() as u32,
            preflop_all_in: own_actions.iter().any(|record| {
                record.statistics.street == TexasStreet::PreFlop
                    && record.statistics.all_in_amount > 0
            }),
            final_stack: state.stack(),
            total_chips: result.final_stacks.iter().sum(),
            match_complete: result.final_stacks.contains(&0),
            first_place: result
                .final_stacks
                .iter()
                .all(|stack| *stack <= state.stack()),
            match_statistics: self.statistics.players[player.index()].clone(),
            ..Default::default()
        };
        let pots = PotStatistics::new(self, result, player, &evaluated);
        report.won_chips = pots.won_chips;
        report.won_pot = pots.won_pot;
        report.won_side_pot_chips = pots.won_side_pot_chips;
        report.tied_winners = pots.tied_winners;
        report.lost_to_lower_unpaired_hand = pots.lost_to_lower_unpaired_hand;
        // The last betting action keeps the street before the remaining board was dealt.
        report.won_all_in_runout = pots.won_each_board
            && self
                .last_action_statistics()
                .is_some_and(|stats| stats.street != TexasStreet::River)
            && own_actions
                .iter()
                .any(|record| record.statistics.all_in_amount > 0);
        report.runout_twice_comeback =
            pots.lost_first_won_second && self.runout_votes().contains(&(player, true));
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
                    record.player.index() == index
                        && record.statistics.street == TexasStreet::PreFlop
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
                && record.statistics.street == TexasStreet::River
                && record.statistics.raised
                && record.statistics.all_in_amount > 0
        }) {
            let subsequent = &self.statistics.history.actions[index + 1..];
            if !subsequent.iter().any(|record| record.statistics.raised) {
                report.river_all_in_folded_categories = subsequent
                    .iter()
                    .filter(|record| {
                        record.action == TexasAction::Fold
                            && record.statistics.street == TexasStreet::River
                    })
                    .filter_map(|record| {
                        evaluated[record.player.index()].map(|hand| hand.category())
                    })
                    .collect();
            }
        }
        Some(report)
    }

    fn river_bet_called(&self, player: PlayerId) -> bool {
        self.statistics
            .history
            .actions
            .iter()
            .enumerate()
            .any(|(index, record)| {
                record.player == player
                    && record.statistics.street == TexasStreet::River
                    && record.statistics.raised
                    && self.statistics.history.actions[index + 1..]
                        .iter()
                        .take_while(|later| !later.statistics.raised)
                        .any(|later| {
                            later.player != player
                                && later.statistics.street == TexasStreet::River
                                && matches!(later.action, TexasAction::Call | TexasAction::AllIn)
                        })
            })
    }
}
