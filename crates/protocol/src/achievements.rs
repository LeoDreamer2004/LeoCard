//! Public achievement messages. Local criteria and progress are not transmitted.

use crate::ProfileId;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AchievementAnnouncement {
    pub profile_id: ProfileId,
    pub name: String,
    pub achievement_id: String,
}
