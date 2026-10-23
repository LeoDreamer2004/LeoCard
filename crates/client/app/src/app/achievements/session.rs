use bevy::prelude::*;
use leocard_achievements::{AchievementDefinition, AchievementTriggerResult};

#[derive(Resource, Default)]
pub(super) struct AchievementSession {
    pub(super) enabled: bool,
    pub(super) dirty: bool,
    pub(super) since_save: f32,
    pub(super) save_failed: bool,
    pub(super) connection_epoch: u64,
    pub(super) connected: bool,
    pub(super) pending: Vec<(&'static AchievementDefinition, Option<u64>)>,
}

#[derive(Resource, Default)]
pub(super) struct AchievementPublication {
    pub(super) unlocked: Vec<String>,
    pub(super) since_send: f32,
}

impl AchievementSession {
    pub(super) fn record(&mut self, result: AchievementTriggerResult, source: Option<u64>) {
        self.dirty |= result.progressed;
        self.pending.extend(
            result
                .unlocked
                .into_iter()
                .map(|definition| (definition, source)),
        );
    }
}
