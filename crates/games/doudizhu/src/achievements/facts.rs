use super::{
    DoudizhuAchievementEvent, DoudizhuAchievementFacts, DoudizhuHandFacts, DoudizhuMatchFacts,
    DoudizhuPlayFacts,
};
use crate::DoudizhuPlayKind;
use leocard_achievements::AchievementTrigger;

pub(super) fn facts(trigger: &AchievementTrigger) -> Option<&DoudizhuAchievementFacts> {
    let (
        player,
        DoudizhuAchievementEvent::ActionAnalyzed {
            player: actor,
            statistics,
        },
    ) = trigger.game::<DoudizhuAchievementEvent>()?;
    (player == *actor).then_some(statistics)
}

pub(super) fn play(
    trigger: &AchievementTrigger,
    predicate: impl FnOnce(&DoudizhuPlayFacts) -> bool,
) -> u64 {
    match facts(trigger) {
        Some(DoudizhuAchievementFacts::Play(facts)) => u64::from(predicate(facts)),
        _ => 0,
    }
}

pub(super) fn hand(
    trigger: &AchievementTrigger,
    predicate: impl FnOnce(&DoudizhuHandFacts) -> bool,
) -> u64 {
    match facts(trigger) {
        Some(DoudizhuAchievementFacts::Hand(facts)) => u64::from(predicate(facts)),
        _ => 0,
    }
}

pub(super) fn whole_match(
    trigger: &AchievementTrigger,
    predicate: impl FnOnce(&DoudizhuMatchFacts) -> bool,
) -> u64 {
    match facts(trigger) {
        Some(DoudizhuAchievementFacts::Match(facts)) => u64::from(predicate(facts)),
        _ => 0,
    }
}

pub(super) fn airplane_groups(facts: &DoudizhuPlayFacts) -> u8 {
    match facts.kind {
        DoudizhuPlayKind::Airplane => facts.card_count / 3,
        DoudizhuPlayKind::AirplaneSingles => facts.card_count / 4,
        DoudizhuPlayKind::AirplanePairs => facts.card_count / 5,
        _ => 0,
    }
}
