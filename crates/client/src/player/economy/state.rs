use super::{ItemId, rewards::login_day, storage::EconomyArchive};
use bevy::prelude::Resource;
use leocard_achievements::AchievementDefinition;
use leocard_protocol::{MatchId, PlayerReferenceChange, ProfileId};
use std::{
    mem,
    time::{self, SystemTime},
};

#[derive(Resource)]
pub struct PlayerEconomy {
    archive: EconomyArchive,
    load_error: Option<String>,
    changes: Vec<i64>,
}

impl PlayerEconomy {
    pub fn load(profile_id: ProfileId) -> Result<Self, String> {
        Ok(Self {
            archive: EconomyArchive::load(profile_id)?,
            load_error: None,
            changes: Vec::new(),
        })
    }

    pub fn unavailable(profile_id: ProfileId, error: String) -> Self {
        Self {
            archive: EconomyArchive::new(profile_id),
            load_error: Some(error),
            changes: Vec::new(),
        }
    }

    pub fn error(&self) -> Option<&str> {
        self.load_error.as_deref()
    }

    pub fn coins(&self) -> u32 {
        self.archive.coins
    }

    /// 只有成功保存的实际余额变化会产生提示，包括购买道具和开发者修改。
    pub fn take_coin_changes(&mut self) -> Vec<i64> {
        mem::take(&mut self.changes)
    }

    pub fn initialize_achievement_rewards(&mut self, earned: &[String]) -> Result<(), String> {
        if self.archive.rewards_initialized {
            return Ok(());
        }
        self.transact(|archive| {
            // 新钱包不补发启用金币奖励之前已经获得的成就。
            archive.rewarded_achievements.extend(earned.iter().cloned());
            archive.rewards_initialized = true;
            Ok(())
        })
    }

    pub fn claim_daily_login(&mut self) -> Result<bool, String> {
        let day = login_day(unix_seconds());
        if self
            .archive
            .last_login_day
            .is_some_and(|previous| previous >= day)
        {
            return Ok(false);
        }
        self.transact(|archive| Ok(archive.reward_daily(day)))
    }

    pub fn reward_achievement(
        &mut self,
        definition: &AchievementDefinition,
    ) -> Result<bool, String> {
        if self.archive.rewarded_achievements.contains(definition.id) {
            return Ok(false);
        }
        self.transact(|archive| Ok(archive.reward_achievement(definition)))
    }

    pub fn settle_match(
        &mut self,
        match_id: MatchId,
        changes: &[PlayerReferenceChange],
    ) -> Result<bool, String> {
        if self.archive.settled_matches.contains(&match_id)
            || !changes
                .iter()
                .any(|change| change.profile_id == self.archive.profile_id)
        {
            return Ok(false);
        }
        self.transact(|archive| Ok(archive.settle_match(match_id, changes)))
    }

    #[cfg(feature = "developer")]
    pub fn set_developer_coins(&mut self, coins: u32) -> Result<(), String> {
        self.transact(|archive| {
            archive.coins = coins;
            Ok(())
        })
    }

    pub fn remaining_seconds(&self, item: ItemId) -> u64 {
        if self.load_error.is_some() {
            return 0;
        }
        self.archive
            .active_items
            .get(&item)
            .copied()
            .unwrap_or(0)
            .saturating_sub(unix_seconds())
    }

    pub fn active(&self, item: ItemId) -> bool {
        self.remaining_seconds(item) > 0
    }

    pub fn buy(&mut self, item: ItemId) -> Result<(), String> {
        if self.active(item) {
            return Err("道具已生效，到期后可再次购买".to_owned());
        }
        let expires = unix_seconds()
            .checked_add(item.definition().duration_seconds)
            .ok_or("道具到期时间超出范围")?;
        self.purchase_until(item, expires)
    }

    pub fn extend(&mut self, item: ItemId) -> Result<(), String> {
        if !self.active(item) {
            return Err("道具已到期，请重新购买".to_owned());
        }
        let expires = self.archive.active_items[&item]
            .checked_add(item.definition().duration_seconds)
            .ok_or("道具到期时间超出范围")?;
        self.purchase_until(item, expires)
    }

    fn purchase_until(&mut self, item: ItemId, expires: u64) -> Result<(), String> {
        self.transact(|archive| {
            archive.coins = archive
                .coins
                .checked_sub(item.definition().price)
                .ok_or("金币不足")?;
            archive.active_items.insert(item, expires);
            Ok(())
        })
    }

    fn transact<T>(
        &mut self,
        change: impl FnOnce(&mut EconomyArchive) -> Result<T, String>,
    ) -> Result<T, String> {
        if let Some(error) = &self.load_error {
            return Err(error.clone());
        }
        let mut updated = self.archive.clone();
        let result = change(&mut updated)?;
        updated.save()?;
        let delta = i64::from(updated.coins) - i64::from(self.archive.coins);
        self.archive = updated;
        if delta != 0 {
            self.changes.push(delta);
        }
        Ok(result)
    }
}

fn unix_seconds() -> u64 {
    SystemTime::now()
        .duration_since(time::UNIX_EPOCH)
        .expect("系统时间早于 Unix 纪元")
        .as_secs()
}
