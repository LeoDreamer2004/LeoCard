use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};
use std::time::{Duration, Instant};

/// Cooperative deadline shared by search and its host. No hidden game state is stored here.
#[derive(Clone, Debug)]
pub struct SearchBudget {
    deadline: Instant,
    cancelled: Arc<AtomicBool>,
}

impl SearchBudget {
    pub fn new(duration: Duration) -> Self {
        Self {
            deadline: Instant::now() + duration,
            cancelled: Arc::new(AtomicBool::new(false)),
        }
    }

    pub fn exhausted(&self) -> bool {
        self.cancelled.load(Ordering::Relaxed) || Instant::now() >= self.deadline
    }

    pub fn cancel(&self) {
        self.cancelled.store(true, Ordering::Relaxed);
    }
}
