pub(super) mod migration;
mod state;

pub use state::{LocalPlayerProfile, PlayerRatingProfile};
use state::{PROFILE_HEADER, StoredGameProfiles, StoredPlayerProfile, StoredRatingProfile};
