use crate::app::achievements::AchievementUnlocked;
use crate::app::runtime::ClientResource;
use bevy::prelude::*;
use leocard_achievements::{ACHIEVEMENT_REGISTRY, AchievementDefinition};
use leocard_client::{LocalPlayerProfile, PlayerAchievements, PlayerEconomy};
use leocard_protocol::{MatchId, PlayerReferenceChange};
use std::collections::VecDeque;

enum Reward {
    Achievement(&'static AchievementDefinition),
    Settlement {
        match_id: MatchId,
        changes: Vec<PlayerReferenceChange>,
    },
}

#[derive(Resource, Default)]
pub(super) struct PendingRewards {
    baseline: Option<Vec<String>>,
    queue: VecDeque<Reward>,
    observed_settlement: Option<MatchId>,
    login_checked: bool,
    retry_after: f32,
    last_error: Option<String>,
}

pub(super) fn setup_rewards(
    profile: Res<LocalPlayerProfile>,
    mut economy: ResMut<PlayerEconomy>,
    mut pending: ResMut<PendingRewards>,
) {
    // 使用已落盘的成就，不读取尚未保存成功的运行中进度。
    if let Ok(book) = PlayerAchievements::load(&profile) {
        let earned = ACHIEVEMENT_REGISTRY
            .iter()
            .filter(|definition| book.earned_at(definition).is_some())
            .collect::<Vec<_>>();
        let baseline = earned
            .iter()
            .map(|definition| definition.id.to_owned())
            .collect::<Vec<_>>();
        // 在运行中成就第一次提交之前落盘，避免新解锁被误当成旧成就。
        if economy.initialize_achievement_rewards(&baseline).is_err() {
            pending.baseline = Some(baseline);
        }
        // 钱包已初始化时，这些记录可恢复成就落盘后尚未发放的奖励。
        pending
            .queue
            .extend(earned.into_iter().map(Reward::Achievement));
    }
}

pub(super) fn process_rewards(
    mut achievements: MessageReader<AchievementUnlocked>,
    client: Option<Res<ClientResource>>,
    mut economy: ResMut<PlayerEconomy>,
    mut pending: ResMut<PendingRewards>,
    time: Res<Time>,
) {
    for event in achievements.read() {
        if let Some(definition) = event.local_definition() {
            pending.queue.push_back(Reward::Achievement(definition));
        }
    }
    if let Some(client) = client
        && let Some((match_id, changes)) = client.0.model().last_finished_match()
        && pending.observed_settlement != Some(match_id)
    {
        pending.observed_settlement = Some(match_id);
        pending.queue.push_back(Reward::Settlement {
            match_id,
            changes: changes.to_vec(),
        });
    }
    pending.retry_after = (pending.retry_after - time.delta_secs()).max(0.0);
    if pending.retry_after > 0.0 || economy.error().is_some() {
        return;
    }
    match pending.apply(&mut economy) {
        Ok(()) => pending.last_error = None,
        Err(error) => {
            if pending.last_error.as_deref() != Some(&error) {
                warn!("{error}；金币结算稍后重试");
            }
            pending.last_error = Some(error);
            pending.retry_after = 0.5;
        }
    }
}

impl PendingRewards {
    fn apply(&mut self, economy: &mut PlayerEconomy) -> Result<(), String> {
        if let Some(baseline) = &self.baseline {
            economy.initialize_achievement_rewards(baseline)?;
            self.baseline = None;
        }
        if !self.login_checked {
            economy.claim_daily_login()?;
            self.login_checked = true;
        }
        // 失败时保留队首并停止：余额有零下限，不能让后来的结算越过它。
        while let Some(reward) = self.queue.front() {
            match reward {
                Reward::Achievement(definition) => {
                    economy.reward_achievement(definition)?;
                }
                Reward::Settlement { match_id, changes } => {
                    economy.settle_match(*match_id, changes)?;
                }
            }
            self.queue.pop_front();
        }
        Ok(())
    }
}
