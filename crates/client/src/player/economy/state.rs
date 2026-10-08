use super::{ItemId, storage::EconomyArchive};
use bevy::prelude::Resource;
use leocard_protocol::ProfileId;
use std::time::{self, SystemTime};

#[derive(Resource)]
pub struct PlayerEconomy {
    archive: EconomyArchive,
    load_error: Option<String>,
}

impl PlayerEconomy {
    pub fn load(profile_id: ProfileId) -> Result<Self, String> {
        Ok(Self {
            archive: EconomyArchive::load(profile_id)?,
            load_error: None,
        })
    }

    pub fn unavailable(profile_id: ProfileId, error: String) -> Self {
        let mut archive = EconomyArchive::new(profile_id);
        archive.coins = 0;
        Self {
            archive,
            load_error: Some(error),
        }
    }

    pub fn error(&self) -> Option<&str> {
        self.load_error.as_deref()
    }

    pub fn coins(&self) -> u32 {
        self.archive.coins
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

    fn transact(
        &mut self,
        change: impl FnOnce(&mut EconomyArchive) -> Result<(), String>,
    ) -> Result<(), String> {
        if let Some(error) = &self.load_error {
            return Err(error.clone());
        }
        let mut updated = self.archive.clone();
        change(&mut updated)?;
        updated.save()?;
        self.archive = updated;
        Ok(())
    }
}

fn unix_seconds() -> u64 {
    SystemTime::now()
        .duration_since(time::UNIX_EPOCH)
        .expect("系统时间早于 Unix 纪元")
        .as_secs()
}
