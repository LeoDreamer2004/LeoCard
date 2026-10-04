use crate::{
    ACHIEVEMENT_REGISTRY, AchievementDefinition, AchievementScope, AchievementTrigger,
    achievement_counts,
};
use leocard_protocol::{AchievementCounts, MatchId};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// A source-assigned event sequence, strictly increasing within one match.
/// Multiple facts in a single revision need distinct sequences. Snapshots must
/// never be assigned a context or submitted as live triggers.
#[derive(Clone, Copy, Debug)]
pub struct AchievementContext {
    pub match_id: MatchId,
    pub hand_index: Option<u32>,
    pub sequence: u128,
}

#[derive(Default, Deserialize, Serialize)]
pub(super) struct AchievementProgress {
    pub(super) criteria: BTreeMap<String, CriterionProgress>,
    pub(super) earned_at: Option<u64>,
}

#[derive(Default, Deserialize, Serialize)]
pub(super) struct CriterionProgress {
    pub(super) count: u64,
    pub(super) reached_at: Option<u64>,
    pub(super) scope: Option<([u8; 16], Option<u32>)>,
}

/// Pure local progress. The caller owns timestamps, persistence and delivery.
#[derive(Default, Deserialize, Serialize)]
pub struct AchievementBook {
    pub(super) progress: BTreeMap<String, AchievementProgress>,
    pub(super) receipts: BTreeMap<[u8; 16], u128>,
}

#[derive(Default)]
pub struct AchievementTriggerResult {
    pub progressed: bool,
    pub unlocked: Vec<&'static AchievementDefinition>,
}

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
