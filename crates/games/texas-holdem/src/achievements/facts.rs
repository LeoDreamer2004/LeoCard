use super::events::TexasAchievementEvent;
use crate::{
    TexasActionStatistics, TexasHandCategory, TexasHandStatistics, TexasRank,
    TexasSquidAwardStatistics,
};
use leocard_achievements::AchievementTrigger;

pub(super) fn hand(
    trigger: &AchievementTrigger,
    predicate: impl FnOnce(&TexasHandStatistics) -> bool,
) -> u64 {
    let Some((
        player,
        TexasAchievementEvent::HandAnalyzed {
            player: actor,
            statistics,
        },
    )) = trigger.game::<TexasAchievementEvent>()
    else {
        return 0;
    };
    u64::from(player == *actor && predicate(statistics))
}

pub(super) fn action(
    trigger: &AchievementTrigger,
    predicate: impl FnOnce(&TexasActionStatistics) -> bool,
) -> u64 {
    let Some((
        player,
        TexasAchievementEvent::ActionApplied {
            player: actor,
            statistics,
            ..
        },
    )) = trigger.game::<TexasAchievementEvent>()
    else {
        return 0;
    };
    u64::from(player == *actor && predicate(statistics))
}

pub(super) fn pair(facts: &TexasHandStatistics, rank: TexasRank) -> bool {
    matches!(facts.hole_cards.as_slice(), [first, second] if first.rank() == rank && second.rank() == rank)
}

pub(super) fn squid(
    trigger: &AchievementTrigger,
    predicate: impl FnOnce(&TexasSquidAwardStatistics) -> bool,
) -> u64 {
    let Some((
        player,
        TexasAchievementEvent::SquidAwarded {
            player: actor,
            statistics,
        },
    )) = trigger.game::<TexasAchievementEvent>()
    else {
        return 0;
    };
    u64::from(player == *actor && predicate(statistics))
}

pub(super) fn offsuit_two_seven(facts: &TexasHandStatistics) -> bool {
    matches!(facts.hole_cards.as_slice(), [first, second] if first.suit() != second.suit() && matches!((first.rank(), second.rank()), (TexasRank::Two, TexasRank::Seven) | (TexasRank::Seven, TexasRank::Two)))
}

pub(super) fn wins(trigger: &AchievementTrigger) -> u64 {
    hand(trigger, |facts| facts.won_chips > 0)
}

pub(super) fn at_least_trips(category: TexasHandCategory, short_deck: bool) -> bool {
    match category {
        TexasHandCategory::HighCard | TexasHandCategory::OnePair | TexasHandCategory::TwoPair => {
            false
        }
        TexasHandCategory::Straight => !short_deck,
        _ => true,
    }
}
