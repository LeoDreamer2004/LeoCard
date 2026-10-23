use bevy::prelude::*;
use leocard_achievements::{AchievementDefinition, AchievementTrigger};

/// Local facts never produce room announcements; their trophy totals still sync.
#[derive(Message)]
pub(crate) struct LocalAchievementTrigger(pub AchievementTrigger);

pub(super) enum AchievementRecipient {
    Local,
    TablePlayer { name: String },
}

#[derive(Message)]
pub(crate) struct AchievementUnlocked {
    pub(super) definition: &'static AchievementDefinition,
    pub(super) recipient: AchievementRecipient,
}

impl AchievementUnlocked {
    pub(crate) fn local_definition(&self) -> Option<&'static AchievementDefinition> {
        matches!(&self.recipient, AchievementRecipient::Local).then_some(self.definition)
    }
}
