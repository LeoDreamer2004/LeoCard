use super::TexasHoldemBotObservation;
use super::equity::equity;
use super::ranges::starting_strength;
use crate::{TexasHoldemAction, TexasHoldemRuleSet, TexasHoldemStreet};
use leocard_game_common::SearchBudget;

pub(super) struct PreflopPolicy<'a> {
    observation: &'a TexasHoldemBotObservation,
    strength: f64,
    raises: usize,
    callers: u32,
}

impl<'a> PreflopPolicy<'a> {
    pub fn new(observation: &'a TexasHoldemBotObservation) -> Self {
        let mut raises = 0;
        let mut callers = 0;
        for record in &observation.history {
            if record.street != TexasHoldemStreet::PreFlop {
                continue;
            }
            match record.action {
                TexasHoldemAction::RaiseTo(_) => raises += 1,
                TexasHoldemAction::AllIn
                    if observation.opponents.iter().any(|other| {
                        other.id == record.player
                            && other.street_commit > TexasHoldemRuleSet::BIG_BLIND
                    }) =>
                {
                    raises += 1
                }
                TexasHoldemAction::Call => callers += 1,
                _ => {}
            }
        }
        Self {
            observation,
            strength: starting_strength(&observation.hole, &observation.rules),
            raises,
            callers,
        }
    }

    pub fn choose(&self, budget: &SearchBudget) -> TexasHoldemAction {
        if self.raises == 0 && self.observation.current_bet <= TexasHoldemRuleSet::BIG_BLIND {
            self.opening()
        } else {
            self.facing_raise(budget)
        }
    }

    fn opening(&self) -> TexasHoldemAction {
        let observation = self.observation;
        let maximum = observation.committed + observation.stack;
        let short_stack = maximum <= TexasHoldemRuleSet::BIG_BLIND * 6;
        let threshold = 0.16 + observation.opponents.len().saturating_sub(1) as f64 * 0.012
            - observation.position * 0.075
            + if short_stack { 0.06 } else { 0.0 }
            - if observation.committed > 0 {
                0.035
            } else {
                0.0
            };
        if observation.call > 0 && self.strength < threshold {
            return TexasHoldemAction::Fold;
        }
        let raise_threshold =
            0.38 + observation.opponents.len() as f64 * 0.012 - observation.position * 0.07;
        if observation.can_raise && self.strength >= raise_threshold {
            if short_stack && self.strength >= 0.4 && maximum > observation.current_bet {
                return TexasHoldemAction::AllIn;
            }
            // Three big blinds plus one per limper. Unacted players are not assumed
            // to call and go to showdown when deciding whether to enter an unopened pot.
            if let Some(action) = self.raise_to(TexasHoldemRuleSet::BIG_BLIND * (3 + self.callers))
            {
                return action;
            }
        }
        self.continue_action()
    }

    fn facing_raise(&self, budget: &SearchBudget) -> TexasHoldemAction {
        let observation = self.observation;
        let opponent_stack = observation
            .opponents
            .iter()
            .map(|other| other.street_commit + other.stack)
            .max()
            .unwrap_or(0);
        let effective = (observation.committed + observation.stack)
            .min(opponent_stack)
            .max(observation.call.max(1));
        let expense = f64::from(observation.call) / f64::from(effective);
        let price = f64::from(observation.call)
            / f64::from((observation.contestable_pot() + observation.call).max(1));
        let size = f64::from(observation.current_bet) / f64::from(TexasHoldemRuleSet::BIG_BLIND);
        let threshold = 0.22
            + ((size - 2.0).max(0.0) * 0.025).min(0.15)
            + self.raises.saturating_sub(1) as f64 * 0.055
            + expense * 0.10
            - observation.position * 0.035
            - if observation.committed > 0 && price < 0.25 {
                0.03
            } else {
                0.0
            }
            - if observation.rules.omaha { 0.035 } else { 0.0 };
        if observation.call > 0 && self.strength < threshold {
            return TexasHoldemAction::Fold;
        }
        let premium = self.strength
            >= 0.68 + self.raises.saturating_sub(1) as f64 * 0.06
                - if observation.rules.omaha { 0.06 } else { 0.0 };
        let affordable = observation.call <= TexasHoldemRuleSet::BIG_BLIND * 3
            && expense <= 0.20
            && self.raises <= 1;
        if affordable && !premium {
            return self.continue_action();
        }
        // Price a call against the players already entering the raised pot.
        // Counting every unacted seat as a showdown opponent severely undervalues it.
        let mut contest = observation.clone();
        contest
            .opponents
            .retain(|other| other.street_commit > TexasHoldemRuleSet::BIG_BLIND || other.all_in);
        let share = equity(&contest, budget) * (0.90 + observation.position * 0.06);
        if observation.call > 0 && share < price + 0.015 {
            return TexasHoldemAction::Fold;
        }
        if premium && observation.can_raise && share > 0.55 {
            if effective <= TexasHoldemRuleSet::BIG_BLIND * 12 || expense >= 0.35 {
                if observation.committed + observation.stack > observation.current_bet {
                    return TexasHoldemAction::AllIn;
                }
            } else if let Some(action) = self.raise_to(observation.current_bet * 3) {
                return action;
            }
        }
        self.continue_action()
    }

    fn continue_action(&self) -> TexasHoldemAction {
        if self.observation.call == 0 {
            TexasHoldemAction::Check
        } else {
            TexasHoldemAction::Call
        }
    }

    fn raise_to(&self, desired: u32) -> Option<TexasHoldemAction> {
        let target = self.observation.raise_sizing().normalize(desired)?;
        Some(
            if target == self.observation.committed + self.observation.stack {
                TexasHoldemAction::AllIn
            } else {
                TexasHoldemAction::RaiseTo(target)
            },
        )
    }
}
