use crate::Delivery;
use std::time::Duration;

/// Advance game-owned automatic operations, deadlines and asynchronous decisions.
pub(crate) trait GameAutomation {
    fn advance_game_time(&mut self, elapsed: Duration) -> Vec<Delivery>;
}
