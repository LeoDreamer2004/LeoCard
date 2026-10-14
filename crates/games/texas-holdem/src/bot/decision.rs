use super::TexasHoldemBotObservation;
use super::equity::equity;
use super::preflop::PreflopPolicy;
use super::ranges::starting_strength;
use crate::{TexasHoldemAction, TexasHoldemStreet};
use leocard_game_common::{SearchBudget, SearchRandom};

pub struct TexasHoldemBot;

impl TexasHoldemBot {
    pub fn choose(
        observation: &TexasHoldemBotObservation,
        budget: &SearchBudget,
    ) -> TexasHoldemAction {
        if let Some(action) = observation.immediate_action() {
            return action;
        }
        if observation.street == TexasHoldemStreet::PreFlop {
            return PreflopPolicy::new(observation).choose(budget);
        }
        let strength = starting_strength(&observation.hole, &observation.rules);
        let opponents = observation.opponents.len() as f64;
        let equity = equity(observation, budget);
        let maximum = observation.committed + observation.stack;
        let pot = f64::from(observation.contestable_pot());
        let call = f64::from(observation.call);
        let realization = if observation.street == TexasHoldemStreet::River {
            1.0
        } else {
            0.93 + observation.position * 0.05
        };
        let call_ev = equity * realization * (pot + call) - call;
        let mut best = if observation.call == 0 {
            TexasHoldemAction::Check
        } else if call_ev > 0.0 {
            TexasHoldemAction::Call
        } else {
            TexasHoldemAction::Fold
        };
        let mut best_ev = call_ev.max(0.0);
        if !observation.can_raise || maximum <= observation.current_bet {
            return best;
        }
        let sizing = observation.raise_sizing();
        let mut targets: Vec<_> = sizing
            .normalize(observation.minimum_raise)
            .into_iter()
            .collect();
        for fraction in [0.4, 0.7, 1.0] {
            let desired = observation.current_bet
                + (f64::from(observation.pot + observation.call) * fraction).round() as u32;
            if let Some(target) = sizing.normalize(desired) {
                targets.push(target);
            }
        }
        if observation.can_all_in && (observation.stack <= observation.pot * 2 || strength > 0.75) {
            targets.push(maximum);
        }
        targets.sort_unstable();
        targets.dedup();
        let mut random = SearchRandom::new(observation.seed().wrapping_add(717));
        let mix = random.unit();
        for target in targets {
            if target > maximum
                || target <= observation.current_bet
                || (!sizing.contains(target) && target != maximum)
            {
                continue;
            }
            let payment = f64::from(target - observation.committed);
            let mut fold_probability = 1.0;
            let mut extra = 0.0;
            for other in &observation.opponents {
                let odds = f64::from(target.saturating_sub(other.street_commit))
                    / f64::from(observation.pot.max(2));
                let fold = if other.all_in {
                    0.0
                } else {
                    (0.12 + odds * 0.18).clamp(0.1, 0.45)
                };
                fold_probability *= fold;
                extra += f64::from(target.saturating_sub(other.street_commit).min(other.stack));
            }
            let called_equity = equity * 0.85;
            let ev = fold_probability * pot
                + (1.0 - fold_probability) * (called_equity * (pot + payment + extra) - payment);
            let value_threshold = 1.0 / (opponents + 1.0) + 0.12;
            let value_bet = equity > value_threshold;
            let bluff = opponents <= 2.0
                && mix < 0.07
                && observation.position > 0.5
                && observation.call == 0;
            if (value_bet || bluff) && ev > best_ev + 0.25 {
                best_ev = ev;
                best = if target == maximum {
                    TexasHoldemAction::AllIn
                } else {
                    TexasHoldemAction::RaiseTo(target)
                };
            }
        }
        best
    }
}
