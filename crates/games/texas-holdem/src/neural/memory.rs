use crate::{TexasAction, TexasGameState, TexasPhase, TexasStreet};
use leocard_ai::TensorMap;

#[derive(Clone, Copy, Debug, Default)]
struct Profile {
    rates: [f32; 5],
    showdowns: u32,
}

/// Cross-hand public observations only. Folded hole cards are never inspected.
#[derive(Clone, Debug, Default)]
pub struct TexasPpoMemory {
    profiles: [Profile; 6],
    last_hand: Option<u32>,
}

impl TexasPpoMemory {
    pub(super) fn features(&self, index: usize) -> [f32; 6] {
        let profile = self.profiles[index];
        let mut output = [0.0; 6];
        output[..5].copy_from_slice(&profile.rates);
        output[5] = (profile.showdowns as f32 / 12.0).min(1.0);
        output
    }

    pub fn observe_completed(&mut self, game: &TexasGameState, chart: &TensorMap) {
        let TexasPhase::Complete(result) = game.phase() else {
            return;
        };
        if game.players().len() != 6 || self.last_hand == Some(game.hand_number()) {
            return;
        }
        self.last_hand = Some(game.hand_number());
        for (index, state) in game.players().iter().enumerate() {
            let profile = &mut self.profiles[index];
            let log: Vec<_> = game
                .betting_history()
                .iter()
                .filter(|record| {
                    record.player.index() == index && record.action != TexasAction::PostBlind
                })
                .collect();
            let aggressive =
                |action| matches!(action, TexasAction::RaiseTo(_) | TexasAction::AllIn);
            update(
                &mut profile.rates[4],
                log.iter().filter(|r| aggressive(r.action)).count() as f32
                    / log.len().max(1) as f32,
            );
            if !result.showdown || state.folded() || result.community.len() != 5 {
                continue;
            }
            profile.showdowns += 1;
            let hand = state.hole_cards();
            let preflop = super::strength::preflop(hand, chart);
            let voluntary = log.iter().any(|r| {
                r.street == TexasStreet::PreFlop
                    && (aggressive(r.action) || r.action == TexasAction::Call)
            });
            update(&mut profile.rates[2], f32::from(voluntary && preflop < 0.4));
            let large = log.iter().any(|r| {
                r.action == TexasAction::AllIn
                    || matches!(r.action, TexasAction::RaiseTo(_)) && r.payment >= r.pot_before
            });
            let strength = super::strength::postflop(hand, &result.community, chart);
            if large {
                update(&mut profile.rates[0], f32::from(strength < 0.45));
                update(&mut profile.rates[1], f32::from(strength > 0.75));
            }
            if log
                .iter()
                .any(|r| r.action == TexasAction::Call && r.payment > 0)
            {
                update(&mut profile.rates[3], f32::from(strength >= 0.5 || large));
            }
        }
    }
}

fn update(rate: &mut f32, value: f32) {
    *rate = *rate * 0.82 + value * 0.18;
}
