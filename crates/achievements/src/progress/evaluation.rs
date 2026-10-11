use super::{AchievementBook, AchievementContext, AchievementTriggerResult, CriterionProgress};
use crate::{
    ACHIEVEMENT_REGISTRY, AchievementDefinition, AchievementScope, AchievementTrigger,
    achievement_counts, registry::derived_triggers,
};
use leocard_protocol::AchievementCounts;

impl AchievementBook {
    pub fn counts(&self) -> AchievementCounts {
        achievement_counts(
            self.progress
                .iter()
                .filter(|(_, value)| value.earned_at.is_some())
                .map(|(id, _)| id.as_str()),
        )
    }

    pub fn earned_at(&self, definition: &AchievementDefinition) -> Option<u64> {
        self.progress.get(definition.id)?.earned_at
    }

    pub fn achieved(&self, definition: &AchievementDefinition) -> bool {
        self.earned_at(definition).is_some()
    }

    pub fn criterion_count(&self, definition: &AchievementDefinition, criterion: &str) -> u64 {
        self.progress
            .get(definition.id)
            .and_then(|entry| entry.criteria.get(criterion))
            .map_or(0, |counter| counter.count)
    }

    /// Unsequenced local events are caller-owned. Boolean criteria are idempotent;
    /// cumulative criteria require a sequenced context when replay is possible.
    pub fn trigger(
        &mut self,
        event: &AchievementTrigger,
        context: Option<AchievementContext>,
        now: u64,
    ) -> AchievementTriggerResult {
        self.trigger_with_registry(event, context, now, ACHIEVEMENT_REGISTRY)
    }

    /// Evaluate a live fact against a supplied registry, including custom signals.
    /// Use one complete registry consistently for a book; criterion IDs must stay stable.
    pub fn trigger_with_registry(
        &mut self,
        event: &AchievementTrigger,
        context: Option<AchievementContext>,
        now: u64,
        definitions: &'static [AchievementDefinition],
    ) -> AchievementTriggerResult {
        let mut result = AchievementTriggerResult::default();
        if let Some(context) = context {
            if self
                .receipts
                .get(&context.match_id.0)
                .is_some_and(|sequence| *sequence >= context.sequence)
            {
                return result;
            }
            self.receipts.insert(context.match_id.0, context.sequence);
            result.progressed = true;
        }
        let (collection, changed) = self.observe_collection(event, now, definitions);
        result.progressed |= changed;
        let event = collection.as_ref().unwrap_or(event);
        result.progressed |= self.reset_streak(event, definitions);
        for definition in definitions {
            if self.achieved(definition) {
                continue;
            }
            for criterion in definition.criteria {
                let scope = match criterion.scope {
                    AchievementScope::Lifetime => None,
                    AchievementScope::Match => {
                        let Some(context) = context else {
                            continue;
                        };
                        Some((context.match_id.0, None))
                    }
                    AchievementScope::Hand => {
                        let Some(context) = context else {
                            continue;
                        };
                        let Some(hand) = context.hand_index else {
                            continue;
                        };
                        Some((context.match_id.0, Some(hand)))
                    }
                };
                if let Some(counter) = self
                    .progress
                    .get_mut(definition.id)
                    .and_then(|entry| entry.criteria.get_mut(criterion.id))
                    && counter.scope != scope
                {
                    *counter = CriterionProgress {
                        scope,
                        ..Default::default()
                    };
                    result.progressed = true;
                }
                let amount = (criterion.amount)(event);
                if amount == 0 {
                    continue;
                }
                let entry = self.progress.entry(definition.id.to_owned()).or_default();
                let counter = entry
                    .criteria
                    .entry(criterion.id.to_owned())
                    .or_insert_with(|| CriterionProgress {
                        scope,
                        ..Default::default()
                    });
                let next = counter.count.saturating_add(amount).min(criterion.target);
                if next != counter.count {
                    counter.count = next;
                    result.progressed = true;
                }
                if counter.count >= criterion.target && counter.reached_at.is_none() {
                    counter.reached_at = Some(now);
                }
            }
            if let Some(entry) = self.progress.get_mut(definition.id)
                && !definition.requirements.is_empty()
                && definition.requirements.iter().all(|group| {
                    group.iter().any(|id| {
                        let Some(criterion) = definition
                            .criteria
                            .iter()
                            .find(|criterion| criterion.id == *id)
                        else {
                            return false;
                        };
                        let expected_scope = match criterion.scope {
                            AchievementScope::Lifetime => None,
                            AchievementScope::Match => {
                                let Some(context) = context else {
                                    return false;
                                };
                                Some((context.match_id.0, None))
                            }
                            AchievementScope::Hand => {
                                let Some(context) = context else {
                                    return false;
                                };
                                let Some(hand) = context.hand_index else {
                                    return false;
                                };
                                Some((context.match_id.0, Some(hand)))
                            }
                        };
                        entry.criteria.get(*id).is_some_and(|counter| {
                            counter.scope == expected_scope && counter.reached_at.is_some()
                        })
                    })
                })
            {
                entry.earned_at = Some(now);
                result.progressed = true;
                result.unlocked.push(definition);
            }
        }
        for derived in derived_triggers(event) {
            let update = self.trigger_with_registry(&derived, None, now, definitions);
            result.progressed |= update.progressed;
            result.unlocked.extend(update.unlocked);
        }
        if !matches!(event, AchievementTrigger::TrophyTotals(_)) {
            let derived = self.trigger_with_registry(
                &AchievementTrigger::TrophyTotals(self.counts()),
                None,
                now,
                definitions,
            );
            result.progressed |= derived.progressed;
            result.unlocked.extend(derived.unlocked);
        }
        result
    }
}
