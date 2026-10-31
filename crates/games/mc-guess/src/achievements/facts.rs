use super::events::McGuessAchievementEvent;
use crate::{
    McGuessActionStatistics, McGuessCatalogue, McGuessEntry, McGuessEntryKind,
    McGuessHandStatistics, McGuessPool,
};
use leocard_achievements::AchievementTrigger;
use std::collections::BTreeSet;
use std::sync::LazyLock;

pub(super) const ENTRY_COLLECTION: &str = "leocard:minecraft_guess/all_entries";
pub(super) const VERSION_COLLECTION: &str = "leocard:minecraft_guess/all_versions";
pub(super) const CONSISTENT_STREAK: &str = "leocard:minecraft_guess/consistent_five";

static VERSIONS: LazyLock<BTreeSet<String>> = LazyLock::new(|| {
    McGuessCatalogue::bundled()
        .entries(McGuessPool::Mixed)
        .filter_map(version)
        .map(str::to_owned)
        .collect()
});

fn version(entry: &McGuessEntry) -> Option<&str> {
    entry
        .attributes
        .iter()
        .find(|attribute| attribute.label == "首次加入版本")?
        .values
        .first()
        .map(String::as_str)
}

pub(super) fn action(
    event: &AchievementTrigger,
    predicate: impl FnOnce(&McGuessActionStatistics) -> bool,
) -> u64 {
    match event.game::<McGuessAchievementEvent>() {
        Some((
            player,
            McGuessAchievementEvent::ActionAnalyzed {
                player: actor,
                statistics,
            },
        )) if player == *actor && player == statistics.player => u64::from(predicate(statistics)),
        _ => 0,
    }
}

pub(super) fn hand(
    event: &AchievementTrigger,
    predicate: impl FnOnce(&McGuessHandStatistics) -> bool,
) -> u64 {
    match event.game::<McGuessAchievementEvent>() {
        Some((
            player,
            McGuessAchievementEvent::HandAnalyzed {
                player: actor,
                statistics,
            },
        )) if player == *actor && player == statistics.player => u64::from(predicate(statistics)),
        _ => 0,
    }
}

pub(super) fn collected(event: &AchievementTrigger, expected: &str) -> u64 {
    match event {
        AchievementTrigger::CollectionCount { id, count, target }
            if *id == expected && *target > 0 =>
        {
            u64::from(count >= target)
        }
        _ => 0,
    }
}

pub(crate) fn derived_triggers(event: &AchievementTrigger) -> Vec<AchievementTrigger> {
    let Some((player, event)) = event.game::<McGuessAchievementEvent>() else {
        return Vec::new();
    };
    match event {
        McGuessAchievementEvent::ActionAnalyzed {
            player: actor,
            statistics,
        } if player == *actor && player == statistics.player => {
            let Some(correct) = &statistics.correct_entry else {
                return Vec::new();
            };
            let catalogue = McGuessCatalogue::bundled();
            let Some(entry) = catalogue.find(correct.kind, &correct.id) else {
                return Vec::new();
            };
            let mut events = Vec::new();
            // The existing collection covers the public block/item catalogue only.
            let collection_kind = match correct.kind {
                McGuessEntryKind::Block => Some("block"),
                McGuessEntryKind::Item => Some("item"),
                _ => None,
            };
            if let Some(kind) = collection_kind {
                events.push(AchievementTrigger::Collection {
                    id: ENTRY_COLLECTION,
                    item: format!("{kind}/{}", correct.id),
                    target: catalogue.entries(McGuessPool::Mixed).count(),
                });
            }
            if let Some(version) = version(entry).filter(|version| VERSIONS.contains(*version)) {
                events.push(AchievementTrigger::Collection {
                    id: VERSION_COLLECTION,
                    item: version.to_owned(),
                    target: VERSIONS.len(),
                });
            }
            events
        }
        McGuessAchievementEvent::HandAnalyzed {
            player: actor,
            statistics,
        } if player == *actor && player == statistics.player => {
            vec![AchievementTrigger::Streak {
                id: CONSISTENT_STREAK,
                qualifies: statistics.consistent,
            }]
        }
        _ => Vec::new(),
    }
}
