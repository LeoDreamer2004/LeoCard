use super::{AchievementBook, CriterionProgress};
use crate::{AchievementDefinition, AchievementTrigger};
use std::collections::btree_map::Entry;

impl AchievementBook {
    pub(super) fn observe_collection(
        &mut self,
        event: &AchievementTrigger,
        now: u64,
        definitions: &[AchievementDefinition],
    ) -> (Option<AchievementTrigger>, bool) {
        let AchievementTrigger::Collection { id, item, target } = event else {
            return (None, false);
        };
        if *target == 0 || !definitions.iter().any(|definition| definition.id == *id) {
            return (None, false);
        }
        let progress = self.progress.entry((*id).to_owned()).or_default();
        if progress.earned_at.is_some() {
            return (None, false);
        }
        let key = format!("unique/{item}");
        let changed = if let Entry::Vacant(entry) = progress.criteria.entry(key) {
            entry.insert(CriterionProgress {
                count: 1,
                reached_at: Some(now),
                scope: None,
            });
            true
        } else {
            false
        };
        let count = progress
            .criteria
            .keys()
            .filter(|key| key.starts_with("unique/"))
            .count();
        (
            Some(AchievementTrigger::CollectionCount {
                id,
                count,
                target: *target,
            }),
            changed,
        )
    }

    pub(super) fn reset_streak(
        &mut self,
        event: &AchievementTrigger,
        definitions: &[AchievementDefinition],
    ) -> bool {
        let AchievementTrigger::Streak {
            id,
            qualifies: false,
        } = event
        else {
            return false;
        };
        if !definitions.iter().any(|definition| definition.id == *id) {
            return false;
        }
        let Some(progress) = self.progress.get_mut(*id) else {
            return false;
        };
        if progress.earned_at.is_some() {
            return false;
        }
        let Some(counter) = progress.criteria.get_mut("progress") else {
            return false;
        };
        if counter.count == 0 {
            return false;
        }
        *counter = CriterionProgress::default();
        true
    }
}
