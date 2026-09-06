use std::collections::HashSet;
use std::fmt;
use std::time::Duration;

use leocard_protocol::{
    AvatarId, ClientCommand, ClientMessage, GameCommand, GameEvent, GameKind, GameRules,
    GameSnapshot, GameViolation, LobbySnapshot, MatchId, PlayerId, PlayerInteraction,
    PlayerInteractionKind, PlayerReferenceChange, ProfileId, RejectReason, RequestId, Revision,
    RoomId, SeatId, ServerEvent, TABLE_SEAT_COUNT, TexasHoldemBlindView, TexasHoldemCommand,
    TexasHoldemEvent, TexasHoldemPhaseView, TexasHoldemPlayerState, TexasHoldemPotAward,
    TexasHoldemProfileStats, TexasHoldemRevealedHand, TexasHoldemSnapshot, TexasHoldemViolation,
};
use leocard_qigui523::reference_point_deltas;
use leocard_texas_holdem::{
    ActionOutcome, GameError, GameState, PassiveBot, Phase, RuleError, TexasHoldemAction,
    TexasHoldemCard, TexasHoldemHandCategory, TexasHoldemPlayerId, TexasHoldemRuleSet, build_deck,
    evaluate_player_hand,
};

use crate::{
    AUTO_PLAY_DELAY, AutoPlayDelayState, ConnectionId, Delivery, HostError, RoomSession,
    new_match_id,
};

mod adapter;
mod automation;
mod commands;
mod lifecycle;
mod snapshot;

pub use adapter::{AdapterError, TablePlayer, TexasHoldemAdapter};

#[cfg(test)]
mod tests;

/// 复用 [`RoomSession`] 的德州扑克权威房间后端。
#[derive(Clone, Debug)]
pub struct TexasHoldemSession {
    room: RoomSession,
    rules: TexasHoldemRuleSet,
    shuffled_deck: Option<Vec<TexasHoldemCard>>,
    game: Option<TexasHoldemAdapter>,
    match_profile_stats: Vec<TexasHoldemProfileStats>,
    finished_reference_changes: Option<Vec<PlayerReferenceChange>>,
    auto_play_delay: Option<AutoPlayDelayState>,
}

fn record_wager(stats: &mut TexasHoldemProfileStats, amount: u32) {
    if amount == 0 {
        return;
    }
    stats.wagered_chips = stats.wagered_chips.saturating_add(u64::from(amount));
    stats.wager_actions = stats.wager_actions.saturating_add(1);
}

const fn hand_category_index(category: TexasHoldemHandCategory) -> usize {
    match category {
        TexasHoldemHandCategory::HighCard => 0,
        TexasHoldemHandCategory::OnePair => 1,
        TexasHoldemHandCategory::TwoPair => 2,
        TexasHoldemHandCategory::ThreeOfAKind => 3,
        TexasHoldemHandCategory::Straight => 4,
        TexasHoldemHandCategory::Flush => 5,
        TexasHoldemHandCategory::FullHouse => 6,
        TexasHoldemHandCategory::FourOfAKind => 7,
        TexasHoldemHandCategory::StraightFlush => 8,
        TexasHoldemHandCategory::RoyalFlush => 9,
    }
}

fn merge_texas_holdem_profile_stats(
    aggregate: &mut TexasHoldemProfileStats,
    current: &TexasHoldemProfileStats,
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
    for (aggregate, current) in aggregate
        .hand_category_counts
        .iter_mut()
        .zip(current.hand_category_counts)
    {
        *aggregate = aggregate.saturating_add(current);
    }
}

fn validate_deck(short_deck: bool, deck: &[TexasHoldemCard]) -> Result<(), HostError> {
    let expected_deck = build_deck(short_deck);
    if deck.len() != expected_deck.len() {
        return Err(HostError::InvalidDeckSize {
            expected: expected_deck.len(),
            actual: deck.len(),
        });
    }
    let expected = expected_deck.into_iter().collect::<HashSet<_>>();
    let actual = deck.iter().copied().collect::<HashSet<_>>();
    if actual.len() != deck.len() || actual != expected {
        return Err(HostError::InvalidDeckContents);
    }
    Ok(())
}
