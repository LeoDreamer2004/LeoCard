use bevy::prelude::*;
use leocard_client::{AchievementDefinition, AchievementTrigger};

/// Local facts never produce room announcements; their trophy totals still sync.
#[derive(Message)]
pub(crate) struct LocalAchievementTrigger(pub AchievementTrigger);

pub(super) enum AchievementRecipient {
    Local,
    TablePlayer { name: String },
}

#[derive(Message)]
pub(super) struct AchievementUnlocked {
    pub(super) definition: &'static AchievementDefinition,
    pub(super) recipient: AchievementRecipient,
}
