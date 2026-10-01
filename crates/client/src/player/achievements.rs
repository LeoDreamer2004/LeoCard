//! Local achievement criteria and unlock dates, starting with the 0.5.0 schema.

use super::{LocalPlayerProfile, config_file};
use bevy::prelude::Resource;
use leocard_achievements::{
    AchievementBook, AchievementContext, AchievementDefinition, AchievementTrigger,
    AchievementTriggerResult,
};
use leocard_protocol::ProfileId;
use serde::{Deserialize, Serialize};
use std::fs;
use std::ops::Deref;
use std::time::{SystemTime, UNIX_EPOCH};

const FILE_HEADER: &[u8] = b"LEOCARD-ACHIEVEMENTS/0.5.0/v3\n";

#[derive(Resource, Deserialize, Serialize)]
pub struct PlayerAchievements {
    profile_id: ProfileId,
    book: AchievementBook,
}

impl Deref for PlayerAchievements {
    type Target = AchievementBook;

    fn deref(&self) -> &Self::Target {
        &self.book
    }
}

impl PlayerAchievements {
    pub fn new(profile_id: ProfileId) -> Self {
        Self {
            profile_id,
            book: AchievementBook::default(),
        }
    }

    pub fn load(profile: &LocalPlayerProfile) -> Result<Self, String> {
        let path =
            config_file("achievements.dat").ok_or_else(|| "无法确定成就档案目录".to_owned())?;
        let bytes = match fs::read(path) {
            Ok(bytes) => bytes,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                return Ok(Self::new(profile.identity.profile_id()));
            }
            Err(error) => return Err(format!("无法读取成就档案：{error}")),
        };
        Self::decode(&bytes, profile.identity.profile_id())
    }

    fn decode(bytes: &[u8], profile_id: ProfileId) -> Result<Self, String> {
        let Some(payload) = bytes.strip_prefix(FILE_HEADER) else {
            // Noncurrent formats deliberately start with an empty book.
            return Ok(Self::new(profile_id));
        };
        let stored: Self =
            postcard::from_bytes(payload).map_err(|error| format!("无法解析成就档案：{error}"))?;
        Ok(if stored.profile_id == profile_id {
            stored
        } else {
            Self::new(profile_id)
        })
    }

    fn encode(&self) -> Result<Vec<u8>, String> {
        let mut bytes = FILE_HEADER.to_vec();
        bytes.extend(
            postcard::to_allocvec(self).map_err(|error| format!("成就档案编码失败：{error}"))?,
        );
        Ok(bytes)
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
            .duration_since(UNIX_EPOCH)
            .map_or(0, |time| time.as_secs());
        self.book.trigger(event, context, now)
    }

    pub fn date_label(&self, achievement: &AchievementDefinition) -> Option<String> {
        let timestamp = self.book.earned_at(achievement)?;
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
        let path =
            config_file("achievements.dat").ok_or_else(|| "无法确定成就档案目录".to_owned())?;
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|error| format!("无法创建成就档案目录：{error}"))?;
        }
        let temporary = path.with_extension("dat.tmp");
        fs::write(&temporary, self.encode()?)
            .map_err(|error| format!("无法保存成就档案：{error}"))?;
        fs::rename(temporary, path).map_err(|error| format!("无法更新成就档案：{error}"))
    }
}
