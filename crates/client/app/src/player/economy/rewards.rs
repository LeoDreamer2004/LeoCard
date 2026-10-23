use super::storage::EconomyArchive;
use leocard_achievements::{AchievementDefinition, AchievementTier};
use leocard_protocol::{MatchId, PlayerReferenceChange};

const DAILY_COINS: i64 = 100;
const COINS_PER_POINT: i64 = 5;

pub(super) fn login_day(unix_seconds: u64) -> u64 {
    // 与成就日期一致，按北京时间的自然日领取。
    unix_seconds.saturating_add(8 * 3600) / 86400
}

impl EconomyArchive {
    pub(super) fn change_coins(&mut self, delta: i64) {
        self.coins = (i64::from(self.coins) + delta).clamp(0, i64::from(u32::MAX)) as u32;
    }

    pub(super) fn reward_daily(&mut self, day: u64) -> bool {
        if self.last_login_day.is_some_and(|previous| previous >= day) {
            return false;
        }
        self.last_login_day = Some(day);
        self.change_coins(DAILY_COINS);
        true
    }

    pub(super) fn reward_achievement(&mut self, definition: &AchievementDefinition) -> bool {
        if !self.rewarded_achievements.insert(definition.id.to_owned()) {
            return false;
        }
        self.change_coins(match definition.tier {
            AchievementTier::Bronze => 50,
            AchievementTier::Silver => 400,
            AchievementTier::Gold => 1500,
        });
        true
    }

    pub(super) fn settle_match(
        &mut self,
        match_id: MatchId,
        changes: &[PlayerReferenceChange],
    ) -> bool {
        if self.settled_matches.contains(&match_id) {
            return false;
        }
        let Some(own) = changes
            .iter()
            .find(|change| change.profile_id == self.profile_id)
        else {
            return false;
        };
        let ticket = changes
            .iter()
            .map(|change| i64::from(change.delta).abs())
            .max()
            .unwrap_or(0)
            / 2;
        self.change_coins(i64::from(own.delta) * COINS_PER_POINT - ticket);
        self.settled_matches.insert(match_id);
        true
    }
}
