use super::{GamePreferences, GlobalPreferences};
use serde::{Deserialize, Serialize};

#[derive(Default, Deserialize, Serialize)]
pub struct SavedPreferences {
    pub global: GlobalPreferences,
    pub games: GamePreferences,
}
