use super::events::QiGuiAchievementEvent;
use crate::QiGuiActionStatistics;
use leocard_achievements::AchievementTrigger;

pub(super) fn facts(trigger: &AchievementTrigger) -> Option<&QiGuiActionStatistics> {
    match trigger.game::<QiGuiAchievementEvent>() {
        Some((
            player,
            QiGuiAchievementEvent::ActionAnalyzed {
                player: actor,
                statistics,
            },
        )) if player == *actor && player.index() == statistics.player.index() => Some(statistics),
        _ => None,
    }
}

pub(super) fn matches(
    trigger: &AchievementTrigger,
    predicate: impl FnOnce(&QiGuiActionStatistics) -> bool,
) -> u64 {
    facts(trigger).map_or(0, |facts| u64::from(predicate(facts)))
}

pub(super) fn amount(
    trigger: &AchievementTrigger,
    count: impl FnOnce(&QiGuiActionStatistics) -> u32,
) -> u64 {
    facts(trigger).map_or(0, |facts| u64::from(count(facts)))
}
