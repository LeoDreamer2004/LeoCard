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

/// The period represented by a result counter is part of its public contract.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ResultScope {
    Hand,
    Match,
}

pub trait GameCountStats {
    const COMPLETION_SCOPE: ResultScope;

    fn completed_games(&self) -> u32;

    fn completed_games_mut(&mut self) -> &mut u32;
}

pub trait ReferenceStats {
    fn total_reference_delta(&self) -> i64;

    fn total_reference_delta_mut(&mut self) -> &mut i64;
}

pub trait PlacementStats {
    const PLACEMENT_SCOPE: ResultScope;

    fn placement_counts(&self) -> &[u32];

    fn placement_counts_mut(&mut self) -> &mut [u32];
}

pub trait ActionTimeProfile {
    fn action_time_stats(&self) -> &ActionTimeStats;

    fn action_time_stats_mut(&mut self) -> &mut ActionTimeStats;

    fn record_action_time(&mut self, millis: u64) {
        self.action_time_stats_mut().record(millis);
    }
}
