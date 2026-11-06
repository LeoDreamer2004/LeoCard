use serde::{Deserialize, Serialize};

/// Cumulative accepted human decisions, owned by each game's long-term profile.
#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct ActionTimeStats {
    pub total_millis: u64,
    pub completed_actions: u64,
    pub longest_millis: u64,
}

impl ActionTimeStats {
    pub fn record(&mut self, millis: u64) {
        self.longest_millis = self.longest_millis.max(millis);
        if let (Some(total), Some(count)) = (
            self.total_millis.checked_add(millis),
            self.completed_actions.checked_add(1),
        ) {
            self.total_millis = total;
            self.completed_actions = count;
        }
    }

    pub fn average_seconds(&self) -> Option<f64> {
        (self.completed_actions > 0)
            .then(|| self.total_millis as f64 / self.completed_actions as f64 / 1000.0)
    }
}
