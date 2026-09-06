mod automation;
mod commands;
mod lifecycle;
mod outcome;
mod room;
mod settlement;
mod snapshot;
mod state;
mod support;
#[cfg(test)]
mod tests;

use crate::{
    AUTO_PLAY_DELAY, AutoPlayDelayState, ConnectionId, Delivery, HostError, RoomSession,
    new_match_id,
};
use outcome::*;
pub use state::UnoSession;
use state::{
    COLOR_ROULETTE_REVEAL_START_DELAY, DRAW_REVEAL_INTERVAL, DRAW_REVEAL_START_DELAY,
    PendingDrawReveal,
};
use std::collections::HashSet;
use std::time::Duration;
use support::*;
