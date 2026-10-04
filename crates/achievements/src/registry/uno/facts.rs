use crate::AchievementTrigger;
use leocard_protocol::{GameEvent, UnoEvent};
use leocard_uno::UnoActionStatistics;

pub(super) fn facts(trigger: &AchievementTrigger) -> Option<&UnoActionStatistics> {
    match trigger {
        AchievementTrigger::Game {
            player,
            event:
                GameEvent::Uno(UnoEvent::ActionAnalyzed {
                    player: actor,
                    statistics,
                }),
        } if player == actor && usize::from(player.0) == statistics.player.0 => Some(statistics),
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
