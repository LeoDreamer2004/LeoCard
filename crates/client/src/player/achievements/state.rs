//! Local achievement progress and unlock date presentation.

use super::storage::AchievementArchive;
use crate::player::LocalPlayerProfile;
use bevy::prelude::Resource;
use leocard_achievements::{
    AchievementBook, AchievementContext, AchievementDefinition, AchievementTrigger,
    AchievementTriggerResult,
};
use leocard_protocol::ProfileId;
use std::{
    ops::Deref,
    time::{self, SystemTime},
};

#[derive(Resource)]
pub struct PlayerAchievements {
    archive: AchievementArchive,
}

impl Deref for PlayerAchievements {
    type Target = AchievementBook;

    fn deref(&self) -> &Self::Target {
        &self.archive.book
    }
}

impl PlayerAchievements {
    pub fn new(profile_id: ProfileId) -> Self {
        Self {
            archive: AchievementArchive::new(profile_id),
        }
    }

    pub fn load(profile: &LocalPlayerProfile) -> Result<Self, String> {
        AchievementArchive::load(profile.identity.profile_id()).map(|archive| Self { archive })
    }

    pub fn trigger(&mut self, event: &AchievementTrigger) -> AchievementTriggerResult {
        self.trigger_with_context(event, None)
    }

    pub fn trigger_with_context(
        &mut self,
        event: &AchievementTrigger,
        context: Option<AchievementContext>,
    ) -> AchievementTriggerResult {
        let now = SystemTime::now()
            .duration_since(time::UNIX_EPOCH)
            .map_or(0, |time| time.as_secs());
        self.archive.book.trigger(event, context, now)
    }

    pub fn date_label(&self, achievement: &AchievementDefinition) -> Option<String> {
        let timestamp = self.archive.book.earned_at(achievement)?;
        // Store UTC seconds; display calendar dates in the Chinese game's UTC+8 zone.
        let days = ((timestamp.saturating_add(8 * 3600)) / 86400) as i64 + 719468;
        let era = days / 146097;
        let day_of_era = days - era * 146097;
        let year_of_era =
            (day_of_era - day_of_era / 1460 + day_of_era / 36524 - day_of_era / 146096) / 365;
        let mut year = year_of_era + era * 400;
        let day_of_year = day_of_era - (365 * year_of_era + year_of_era / 4 - year_of_era / 100);
        let month_index = (5 * day_of_year + 2) / 153;
        let day = day_of_year - (153 * month_index + 2) / 5 + 1;
        let month = month_index + if month_index < 10 { 3 } else { -9 };
        year += i64::from(month <= 2);
        Some(format!("{year:04}.{month:02}.{day:02}"))
    }

    pub fn save(&self) -> Result<(), String> {
        self.archive.save()
    }
}
