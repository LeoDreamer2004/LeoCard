use crate::action_timing::PendingAction;
use leocard_protocol::{GameCommand, GameSnapshot};

/// Human decision boundaries only. Rule deadlines and playback clocks are separate.
pub(crate) trait DecisionTiming {
    fn pending_actions(&self, snapshot: &GameSnapshot) -> Vec<PendingAction>;

    fn is_decision(command: &GameCommand) -> bool;
}
