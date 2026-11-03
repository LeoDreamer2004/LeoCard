use super::super::ShengjiBotObservation;
use super::super::hand::{category_index, face};
use super::super::knowledge::PublicKnowledge;
use super::super::sampling::DealSampler;
use super::super::simulation::Simulation;
use super::evaluation::BurialProblem;
use super::plans::{Candidate, rank};
use crate::{ShengjiCard, ShengjiRuleSet, ShengjiTeamId, ShengjiTrump};
use leocard_game_common::{SearchBudget, SearchRandom};
use std::time::{Duration, Instant};

pub(in crate::bot) fn bury(
    observation: &ShengjiBotObservation,
    budget: &SearchBudget,
    report: &mut impl FnMut(Vec<ShengjiCard>),
) -> Vec<ShengjiCard> {
    let problem = BurialProblem::new(
        &observation.hand,
        observation.rules.kitty_size(),
        observation.trump,
        observation.rules,
        ShengjiTeamId::of(observation.player) == observation.collecting_team,
    );
    let mut candidates = problem.plans();
    report(candidates[0].cards.clone());
    improve(&problem, &mut candidates, budget, report);
    let choices = shortlist(&problem, &candidates);
    // Future copies/exchanges can change the trump or holdings. Do not simulate
    // those rule variants as though the current burial immediately started play.
    if observation.rules.bottom_copy || observation.rules.five_trump_crossing || budget.exhausted()
    {
        return choices[0].cards.clone();
    }
    compare(observation, &choices, budget, report)
}

pub(in crate::bot) fn return_cards(
    hand: &[ShengjiCard],
    count: usize,
    trump: ShengjiTrump,
    rules: ShengjiRuleSet,
    budget: &SearchBudget,
) -> Vec<ShengjiCard> {
    let problem = BurialProblem::new(hand, count, trump, rules, false);
    let mut candidates = problem.plans();
    improve(&problem, &mut candidates, budget, &mut |_| {});
    candidates.remove(0).cards
}

fn improve(
    problem: &BurialProblem<'_>,
    candidates: &mut Vec<Candidate>,
    budget: &SearchBudget,
    report: &mut impl FnMut(Vec<ShengjiCard>),
) {
    // Refine several distinct plans. Whole pairs/triples/quads can be replaced
    // together, so an intermediate broken pair need not improve the evaluation.
    let began = Instant::now();
    let starts = shortlist(problem, candidates);
    for mut selected in starts {
        for _ in 0..3 {
            let kept = problem.kept(&selected.cards);
            let outgoing = groups(&selected.cards);
            let incoming = groups(&kept);
            let mut next = selected.clone();
            for remove in &outgoing {
                for add in incoming.iter().filter(|add| add.len() == remove.len()) {
                    if budget.exhausted() || began.elapsed() >= Duration::from_millis(120) {
                        rank(candidates);
                        return;
                    }
                    let mut cards = selected.cards.clone();
                    cards.retain(|card| !remove.contains(card));
                    cards.extend(add);
                    let candidate = problem.candidate(cards);
                    if candidate.value > next.value + 0.001 {
                        next = candidate;
                        // Publish immediately: interruption halfway through the
                        // neighborhood must not discard an improvement already found.
                        if next.value > candidates[0].value + 0.001 {
                            candidates.push(next.clone());
                            rank(candidates);
                            report(candidates[0].cards.clone());
                        }
                    }
                }
            }
            if next.value <= selected.value + 0.001 {
                break;
            }
            selected = next;
            if selected.value > candidates[0].value {
                report(selected.cards.clone());
            }
            candidates.push(selected.clone());
            rank(candidates);
        }
    }
}

fn groups(cards: &[ShengjiCard]) -> Vec<Vec<ShengjiCard>> {
    let mut groups: Vec<_> = cards.iter().map(|card| vec![*card]).collect();
    for (index, card) in cards.iter().enumerate() {
        if cards[..index].iter().any(|other| other.same_face(*card)) {
            continue;
        }
        let copies: Vec<_> = cards
            .iter()
            .copied()
            .filter(|other| other.same_face(*card))
            .collect();
        for width in 2..=copies.len() {
            groups.push(copies[..width].to_vec());
        }
    }
    groups
}

fn shortlist(problem: &BurialProblem<'_>, candidates: &[Candidate]) -> Vec<Candidate> {
    let mut choices = Vec::new();
    let mut shapes = Vec::new();
    for candidate in candidates {
        let mut shape = [0_u8; 5];
        for card in &candidate.cards {
            shape[category_index(*card, problem.trump)] += 1;
        }
        if choices.len() < 2 || !shapes.contains(&shape) {
            choices.push(candidate.clone());
            shapes.push(shape);
        }
        if choices.len() == 6 {
            break;
        }
    }
    choices
}

fn prepared(observation: &ShengjiBotObservation, candidate: &Candidate) -> ShengjiBotObservation {
    let mut next = observation.clone();
    next.hand.retain(|card| !candidate.cards.contains(card));
    next.lengths[usize::from(next.player.0)] = next.hand.len();
    next.bottom = Some(candidate.cards.clone());
    next
}

fn compare(
    observation: &ShengjiBotObservation,
    choices: &[Candidate],
    budget: &SearchBudget,
    report: &mut impl FnMut(Vec<ShengjiCard>),
) -> Vec<ShengjiCard> {
    let mut best = choices[0].cards.clone();
    if choices.len() == 1 {
        return best;
    }
    let observations: Vec<_> = choices
        .iter()
        .map(|choice| prepared(observation, choice))
        .collect();
    let public = PublicKnowledge::from_observation(observation);
    let sampler = DealSampler::new(&observations[0], &public);
    let mut seed = u64::from(observation.player.0);
    let mut hand = observation.hand.clone();
    hand.sort_unstable_by_key(|card| (face(*card), card.deck()));
    for card in hand {
        seed = seed.wrapping_mul(59).wrapping_add((face(card) + 1) as u64);
    }
    let mut random = SearchRandom::new(seed);
    let mut totals = vec![0.0; choices.len()];
    let mut samples = 0_u32;
    for _ in 0..24 {
        let Some(deal) = sampler.sample(&mut random, budget) else {
            break;
        };
        let values: Option<Vec<_>> = observations
            .iter()
            .map(|next| {
                let mut deal = deal.clone();
                deal.hands[usize::from(next.player.0)] = next.hand.clone();
                deal.bottom = next.bottom.clone().expect("burial");
                let mut position = Simulation::new(next, deal, &public);
                position.actor = observation.dealer;
                position.rollout(observation.player, budget)
            })
            .collect();
        // Compare every candidate against identical opponents. Discard the whole
        // sample if any rollout times out, rather than favoring early candidates.
        let Some(values) = values else {
            break;
        };
        samples += 1;
        for (total, value) in totals.iter_mut().zip(values) {
            *total += value;
        }
        if samples < 2 {
            continue;
        }
        let weight = 0.15 / f64::from(samples).sqrt();
        let index = totals
            .iter()
            .enumerate()
            .max_by(|(a, av), (b, bv)| {
                let score = |i: usize, v: f64| v / f64::from(samples) + choices[i].value * weight;
                score(*a, **av).total_cmp(&score(*b, **bv))
            })
            .expect("plans")
            .0;
        best = choices[index].cards.clone();
        report(best.clone());
    }
    best
}
