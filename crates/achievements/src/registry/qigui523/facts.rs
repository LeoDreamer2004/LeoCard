use crate::AchievementTrigger;
use leocard_protocol::{GameEvent, QiGui523Event};
use leocard_qigui523::QiGuiActionStatistics;

pub(super) fn facts(trigger: &AchievementTrigger) -> Option<&QiGuiActionStatistics> {
    match trigger {
        AchievementTrigger::Game {
            player,
            event:
                GameEvent::QiGui523(QiGui523Event::ActionAnalyzed {
                    player: actor,
                    statistics,
                }),
        } if player == actor && usize::from(player.0) == statistics.player.0 => Some(statistics),
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
