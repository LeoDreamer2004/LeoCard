use super::events::TexasAchievementEvent;
use crate::{
    SquidAwardStatistics, TexasHoldemActionStatistics, TexasHoldemHandCategory as Category,
    TexasHoldemHandStatistics, TexasHoldemRank,
};
use leocard_achievements::AchievementTrigger;

pub(super) fn hand(
    trigger: &AchievementTrigger,
    predicate: impl FnOnce(&TexasHoldemHandStatistics) -> bool,
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
    predicate: impl FnOnce(&TexasHoldemActionStatistics) -> bool,
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

pub(super) fn pair(facts: &TexasHoldemHandStatistics, rank: TexasHoldemRank) -> bool {
    matches!(facts.hole_cards.as_slice(), [first, second] if first.rank() == rank && second.rank() == rank)
}

pub(super) fn squid(
    trigger: &AchievementTrigger,
    predicate: impl FnOnce(&SquidAwardStatistics) -> bool,
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

pub(super) fn offsuit_two_seven(facts: &TexasHoldemHandStatistics) -> bool {
    matches!(facts.hole_cards.as_slice(), [first, second] if first.suit() != second.suit() && matches!((first.rank(), second.rank()), (TexasHoldemRank::Two, TexasHoldemRank::Seven) | (TexasHoldemRank::Seven, TexasHoldemRank::Two)))
}

pub(super) fn wins(trigger: &AchievementTrigger) -> u64 {
    hand(trigger, |facts| facts.won_chips > 0)
}

pub(super) fn at_least_trips(category: Category, short_deck: bool) -> bool {
    match category {
        Category::HighCard | Category::OnePair | Category::TwoPair => false,
        Category::Straight => !short_deck,
        _ => true,
    }
}
