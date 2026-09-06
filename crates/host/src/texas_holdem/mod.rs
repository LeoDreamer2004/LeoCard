mod adapter;
mod automation;
mod commands;
mod lifecycle;
mod snapshot;
mod state;
mod support;
#[cfg(test)]
mod tests;

use crate::{
    AUTO_PLAY_DELAY, AutoPlayDelayState, ConnectionId, Delivery, HostError, RoomSession,
    new_match_id,
};
pub use adapter::{AdapterError, TablePlayer, TexasHoldemAdapter};
pub use state::TexasHoldemSession;
use std::collections::HashSet;
use std::fmt;
use std::time::Duration;
use support::{hand_category_index, merge_texas_holdem_profile_stats, record_wager, validate_deck};
