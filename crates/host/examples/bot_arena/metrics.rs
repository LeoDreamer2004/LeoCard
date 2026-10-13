use leocard_game_common::SearchBudget;
use std::time::{Duration, Instant};

#[derive(Default)]
pub struct Metrics {
    pub decisions: Vec<Duration>,
    pub wins: usize,
    pub draws: usize,
    pub score: i64,
    pub gains: usize,
    pub calls: usize,
    pub folds: usize,
    pub preflop_actions: usize,
    pub preflop_folds: usize,
    pub flops: usize,
    pub off_step_raises: usize,
}

impl Metrics {
    pub fn decide<A>(&mut self, duration: Duration, choose: impl FnOnce(&SearchBudget) -> A) -> A {
        let start = Instant::now();
        let result = choose(&SearchBudget::new(duration));
        let elapsed = start.elapsed();
        assert!(
            elapsed < Duration::from_secs(5),
            "decision exceeded 5 seconds: {elapsed:?}"
        );
        self.decisions.push(elapsed);
        result
    }

    pub fn report(&mut self, game: &str) {
        self.decisions.sort_unstable();
        let count = self.decisions.len();
        let mean = self
            .decisions
            .iter()
            .map(Duration::as_secs_f64)
            .sum::<f64>()
            / count.max(1) as f64;
        let p95 = self
            .decisions
            .get(count.saturating_sub(1) * 95 / 100)
            .copied()
            .unwrap_or_default();
        println!(
            "{game}: decisions={count}, wins={}, draws={}, score={}, raises/calls/folds={}/{}/{}, mean={mean:.4}s p95={p95:?} max={:?}",
            self.wins,
            self.draws,
            self.score,
            self.gains,
            self.calls,
            self.folds,
            self.decisions.last()
        );
        if game == "texas" {
            println!(
                "preflop folds={}/{} ({:.1}%), hands reaching flop={}, off-step raises={}",
                self.preflop_folds,
                self.preflop_actions,
                self.preflop_folds as f64 / self.preflop_actions.max(1) as f64 * 100.0,
                self.flops,
                self.off_step_raises,
            );
        }
    }
}
