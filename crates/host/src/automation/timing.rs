use leocard_game_common::PlayerId;
use std::time::Duration;

pub(crate) const AUTOMATIC_ACTION_DELAY: Duration = Duration::from_secs(1);

#[derive(Clone, Debug)]
pub(crate) struct AutomaticActionDelayState {
    pub player: PlayerId,
    pub remaining: Duration,
}
