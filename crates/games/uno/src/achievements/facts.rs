use super::events::UnoAchievementEvent;
use crate::UnoActionStatistics;
use leocard_achievements::AchievementTrigger;

pub(super) fn facts(trigger: &AchievementTrigger) -> Option<&UnoActionStatistics> {
    match trigger.game::<UnoAchievementEvent>() {
        Some((
            player,
            UnoAchievementEvent::ActionAnalyzed {
                player: actor,
                statistics,
            },
        )) if player == *actor && player.index() == statistics.player.index() => Some(statistics),
        _ => None,
    }
}

pub(super) fn matches(
    trigger: &AchievementTrigger,
    predicate: impl FnOnce(&UnoActionStatistics) -> bool,
) -> u64 {
    facts(trigger).map_or(0, |facts| u64::from(predicate(facts)))
}

pub(super) fn amount(
    trigger: &AchievementTrigger,
    count: impl FnOnce(&UnoActionStatistics) -> u32,
) -> u64 {
    facts(trigger).map_or(0, |facts| u64::from(count(facts)))
}
