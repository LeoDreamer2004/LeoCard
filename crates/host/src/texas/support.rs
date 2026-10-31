use leocard_texas_holdem::{
    TexasCard, TexasGameError, TexasHandCategory, TexasProfileStats, build_deck,
};

use std::collections::HashSet;

pub(super) fn record_wager(stats: &mut TexasProfileStats, amount: u32) {
    if amount == 0 {
        return;
    }
    stats.wagered_chips = stats.wagered_chips.saturating_add(u64::from(amount));
    stats.wager_actions = stats.wager_actions.saturating_add(1);
}

pub(super) const fn hand_category_index(category: TexasHandCategory) -> usize {
    match category {
        TexasHandCategory::HighCard => 0,
        TexasHandCategory::OnePair => 1,
        TexasHandCategory::TwoPair => 2,
        TexasHandCategory::ThreeOfAKind => 3,
        TexasHandCategory::Straight => 4,
        TexasHandCategory::Flush => 5,
        TexasHandCategory::FullHouse => 6,
        TexasHandCategory::FourOfAKind => 7,
        TexasHandCategory::StraightFlush => 8,
        TexasHandCategory::RoyalFlush => 9,
    }
}

pub(super) fn merge_texas_profile_stats(
    aggregate: &mut TexasProfileStats,
    current: &TexasProfileStats,
) {
    aggregate.wagered_chips = aggregate
        .wagered_chips
        .saturating_add(current.wagered_chips);
    aggregate.wager_actions = aggregate
        .wager_actions
        .saturating_add(current.wager_actions);
    aggregate.voluntary_actions = aggregate
        .voluntary_actions
        .saturating_add(current.voluntary_actions);
    aggregate.check_actions = aggregate
        .check_actions
        .saturating_add(current.check_actions);
    aggregate.raise_actions = aggregate
        .raise_actions
        .saturating_add(current.raise_actions);
    aggregate.all_in_actions = aggregate
        .all_in_actions
        .saturating_add(current.all_in_actions);
    aggregate.hands_played = aggregate.hands_played.saturating_add(current.hands_played);
    aggregate.hands_folded = aggregate.hands_folded.saturating_add(current.hands_folded);
    aggregate.entry_tracked_hands = aggregate
        .entry_tracked_hands
        .saturating_add(current.entry_tracked_hands);
    aggregate.entered_hands = aggregate
        .entered_hands
        .saturating_add(current.entered_hands);
    for (aggregate, current) in aggregate
        .hand_category_counts
        .iter_mut()
        .zip(current.hand_category_counts)
    {
        *aggregate = aggregate.saturating_add(current);
    }
}

pub(super) fn validate_deck(short_deck: bool, deck: &[TexasCard]) -> Result<(), TexasGameError> {
    let expected_deck = build_deck(short_deck);
    if deck.len() != expected_deck.len() {
        return Err(TexasGameError::InvalidDeckSize {
            expected: expected_deck.len(),
            actual: deck.len(),
        });
    }
    let expected = expected_deck.into_iter().collect::<HashSet<_>>();
    let actual = deck.iter().copied().collect::<HashSet<_>>();
    if actual.len() != deck.len() || actual != expected {
        return Err(TexasGameError::InvalidDeckContents);
    }
    Ok(())
}
