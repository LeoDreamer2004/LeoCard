use super::FourPlayerObservation;
use super::evaluation::HandEvaluator;
use super::policy::{Candidate, PublicTurn, shortlist};
use crate::{DoudizhuAction, DoudizhuGameState, DoudizhuPhase};
use leocard_game_common::{PlayerId, SearchBudget, SearchRandom};
use std::time::{Duration, Instant};

const SEARCH_TIME: Duration = Duration::from_millis(600);
const MAX_SAMPLES: usize = 24;
const ROLLOUT_PLIES: usize = 48;

pub struct FourPlayerBot;

impl FourPlayerBot {
    pub fn fallback(input: &FourPlayerObservation) -> DoudizhuAction {
        let mut evaluator = HandEvaluator::new(input.rules);
        root_candidates(input, &mut evaluator).remove(0).action
    }

    pub fn choose_with_progress(
        input: &FourPlayerObservation,
        budget: &SearchBudget,
        report: &mut dyn FnMut(DoudizhuAction),
    ) -> DoudizhuAction {
        let started = Instant::now();
        let mut evaluator = HandEvaluator::new(input.rules);
        let candidates = root_candidates(input, &mut evaluator);
        let mut best = candidates[0].action.clone();
        report(best.clone());
        if candidates.len() == 1 || candidates[0].cost < -999.0 {
            return best;
        }
        let stopped = || budget.exhausted() || started.elapsed() >= SEARCH_TIME;
        let mut random = SearchRandom::new(input.seed());
        // Small heuristic priors stabilize decisions when only a few sweeps fit the budget.
        let mut totals: Vec<_> = candidates
            .iter()
            .map(|candidate| -((candidate.cost - candidates[0].cost) * 0.08).min(0.6))
            .collect();
        for _ in 0..MAX_SAMPLES {
            if stopped() {
                break;
            }
            let deal = input.sample(&mut random);
            let seed = random.next_u64();
            let mut scores = Vec::new();
            for candidate in &candidates {
                if stopped() {
                    return best;
                }
                let mut game = deal.clone();
                game.act(input.actor, candidate.action.clone())
                    .expect("legal candidate in sampled deal");
                let Some(score) = rollout(game, input.actor, &mut evaluator, seed, &stopped) else {
                    return best;
                };
                scores.push(score);
            }
            // Commit only complete sweeps: every candidate sees the same sampled hands.
            for (total, score) in totals.iter_mut().zip(scores) {
                *total += score;
            }
            let index = totals
                .iter()
                .enumerate()
                .max_by(|(_, left), (_, right)| left.total_cmp(right))
                .expect("nonempty shortlist")
                .0;
            best = candidates[index].action.clone();
            report(best.clone());
        }
        best
    }
}

fn root_candidates(input: &FourPlayerObservation, evaluator: &mut HandEvaluator) -> Vec<Candidate> {
    shortlist(
        PublicTurn {
            actor: input.actor,
            landlord: input.landlord,
            counts: &input.counts,
            table: input.table.as_ref(),
            rules: input.rules,
        }
        .candidates(&input.hand, evaluator),
    )
}

fn rollout(
    mut game: DoudizhuGameState,
    observer: PlayerId,
    evaluator: &mut HandEvaluator,
    seed: u64,
    stopped: &impl Fn() -> bool,
) -> Option<f64> {
    let mut random = SearchRandom::new(seed);
    for _ in 0..ROLLOUT_PLIES {
        if stopped() {
            return None;
        }
        if game.phase == DoudizhuPhase::Finished {
            return Some(value(&game, observer, evaluator));
        }
        let counts: Vec<_> = game.hand_lengths().into_iter().map(usize::from).collect();
        // Each simulated actor uses only its own sampled hand and public table/counts.
        let candidates = PublicTurn {
            actor: game.current,
            landlord: game.landlord.expect("playing position"),
            counts: &counts,
            table: game.table.as_ref(),
            rules: game.rules,
        }
        .candidates(game.hand(game.current).expect("actor"), evaluator);
        // Occasionally try a nearly tied move rather than repeating one deterministic line.
        let index = if candidates.len() > 1
            && candidates[0].cost > -999.0
            && candidates[1].cost - candidates[0].cost < 0.35
            && random.index(5) == 0
        {
            1
        } else {
            0
        };
        game.act(game.current, candidates[index].action.clone())
            .expect("legal rollout action");
    }
    Some(value(&game, observer, evaluator))
}

fn value(game: &DoudizhuGameState, observer: PlayerId, evaluator: &mut HandEvaluator) -> f64 {
    let landlord = game.landlord.expect("sampled landlord");
    let landlord_view = observer == landlord;
    if let Some(result) = &game.result {
        return if result.landlord_won == landlord_view {
            1.0
        } else {
            -1.0
        };
    }
    // These are generated sample hands, never the authoritative hidden hands.
    let landlord_cost = evaluator.cost(game.hand(landlord).expect("landlord"));
    let farmer_cost = (0..4)
        .map(PlayerId)
        .filter(|player| *player != landlord)
        .map(|player| evaluator.cost(game.hand(player).expect("farmer")))
        .fold(f64::INFINITY, f64::min);
    let initiative = if game.current == landlord { 0.2 } else { -0.2 };
    let score = ((farmer_cost - landlord_cost + initiative) / 3.0).tanh() * 0.9;
    if landlord_view { score } else { -score }
}
