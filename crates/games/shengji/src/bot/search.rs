use super::ShengjiBotObservation;
use super::endgame;
use super::evaluation::{Evaluation, PositionView};
use super::hand::face;
use super::knowledge::PublicKnowledge;
use super::sampling::DealSampler;
use super::simulation::Simulation;
use crate::ShengjiCard;
use leocard_game_common::{SearchBudget, SearchRandom};

pub(super) fn choose_play(
    observation: &ShengjiBotObservation,
    budget: &SearchBudget,
) -> Vec<ShengjiCard> {
    let public = PublicKnowledge::from_observation(observation);
    let evaluation = Evaluation::new(PositionView {
        player: observation.player,
        hand: &observation.hand,
        trump: observation.trump,
        rules: observation.rules,
        trick: observation.trick.as_ref(),
        public: &public,
        lengths: &observation.lengths,
        collecting_score: observation.collecting_score,
        bottom: observation.bottom.as_deref(),
    });
    let choices = evaluation.ranked(16);
    let mut best = choices[0].0.cards.clone();
    if choices.len() == 1 || budget.exhausted() {
        return best;
    }
    let sampler = DealSampler::new(observation, &public);
    let mut random = SearchRandom::new(observation_seed(observation));
    let mut totals = vec![0.0; choices.len()];
    let mut samples = 0_u32;
    for _ in 0..64 {
        let Some(deal) = sampler.sample(&mut random, budget) else {
            break;
        };
        let position = Simulation::new(observation, deal, &public);
        let continuations: Vec<_> = choices
            .iter()
            .map(|(play, _)| {
                let mut next = position.clone();
                next.play(play.clone());
                next
            })
            .collect();
        let mut nodes = 12_000;
        let exact: Option<Vec<_>> = if position.hands.iter().all(|hand| hand.len() <= 3) {
            continuations
                .iter()
                .map(|next| endgame::solve(next, observation.player, budget, &mut nodes))
                .collect()
        } else {
            None
        };
        let values = exact.or_else(|| {
            continuations
                .into_iter()
                .map(|next| next.rollout(observation.player, budget))
                .collect::<Option<Vec<_>>>()
        });
        let Some(values) = values else { break };
        // Only commit completed paired comparisons: a timeout may not give an
        // early candidate more samples, or mix exact and approximate outcomes.
        samples += 1;
        for (total, value) in totals.iter_mut().zip(values) {
            *total += value;
        }
        if samples < 4 {
            continue;
        }
        let prior_weight = 0.25 / f64::from(samples).sqrt();
        let index = totals
            .iter()
            .enumerate()
            .max_by(|(left_index, left), (right_index, right)| {
                let score = |index: usize, sum: f64| {
                    sum / f64::from(samples) + choices[index].1 * prior_weight
                };
                score(*left_index, **left).total_cmp(&score(*right_index, **right))
            })
            .expect("candidates")
            .0;
        best = choices[index].0.cards.clone();
        if budget.exhausted() {
            break;
        }
    }
    best
}

fn observation_seed(observation: &ShengjiBotObservation) -> u64 {
    let mut seed = u64::from(observation.player.0) + u64::from(observation.collecting_score);
    for card in observation.hand.iter().chain(
        observation
            .history
            .iter()
            .chain(observation.trick.iter())
            .flat_map(|trick| trick.plays.iter().flat_map(|(_, play)| &play.cards)),
    ) {
        seed = seed.wrapping_mul(59).wrapping_add((face(*card) + 1) as u64);
    }
    seed
}
