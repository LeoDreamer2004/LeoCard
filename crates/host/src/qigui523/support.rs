use crate::HostError;
use leocard_protocol::{PlayerId, QiGui523ProfileStats, RuleViolation};
use leocard_qigui523::{GameError, PlayError, QiGuiCard, QiGuiPlayerId, build_deck};
use std::{collections::HashSet, time::Duration};

pub(super) fn validate_deck(deck_count: u8, deck: &[QiGuiCard]) -> Result<(), HostError> {
    let expected_deck = build_deck(deck_count);
    if deck.len() != expected_deck.len() {
        return Err(HostError::InvalidDeckSize {
            expected: expected_deck.len(),
            actual: deck.len(),
        });
    }
    let expected: HashSet<_> = expected_deck.into_iter().collect();
    let actual: HashSet<_> = deck.iter().copied().collect();
    if actual.len() != deck.len() || actual != expected {
        return Err(HostError::InvalidDeckContents);
    }
    Ok(())
}

pub(super) fn merge_qigui523_play_stats(
    aggregate: &mut QiGui523ProfileStats,
    current: &QiGui523ProfileStats,
) {
    aggregate.straight_plays = aggregate
        .straight_plays
        .saturating_add(current.straight_plays);
    aggregate.consecutive_pair_plays = aggregate
        .consecutive_pair_plays
        .saturating_add(current.consecutive_pair_plays);
    aggregate.airplane_plays = aggregate
        .airplane_plays
        .saturating_add(current.airplane_plays);
    aggregate.bomb_plays = aggregate.bomb_plays.saturating_add(current.bomb_plays);
    aggregate.heaven_bomb_plays = aggregate
        .heaven_bomb_plays
        .saturating_add(current.heaven_bomb_plays);
    aggregate.longest_straight = aggregate.longest_straight.max(current.longest_straight);
    aggregate.longest_consecutive_pairs = aggregate
        .longest_consecutive_pairs
        .max(current.longest_consecutive_pairs);
    aggregate.longest_airplane = aggregate.longest_airplane.max(current.longest_airplane);
}

pub(super) fn to_core_player(player: PlayerId) -> QiGuiPlayerId {
    QiGuiPlayerId(usize::from(player.0))
}

pub(super) fn from_core_player(player: QiGuiPlayerId) -> PlayerId {
    PlayerId(u8::try_from(player.0).expect("core supports at most six players"))
}

pub(super) fn map_game_error(error: &GameError) -> RuleViolation {
    match error {
        GameError::InvalidPlayer(_) => RuleViolation::InvalidPlayer,
        GameError::NotPlayersTurn { .. } => RuleViolation::NotPlayersTurn,
        GameError::GameAlreadyFinished => RuleViolation::GameAlreadyFinished,
        GameError::MustLeadWithCards => RuleViolation::MustLeadWithCards,
        GameError::CardNotInHand(_) => RuleViolation::CardNotInHand,
        GameError::InvalidPlay(
            PlayError::Empty
            | PlayError::DuplicatePhysicalCard(_)
            | PlayError::CardOutsideConfiguredDeck(_)
            | PlayError::InvalidPattern,
        )
        | GameError::InvalidRules(_)
        | GameError::InvalidDeckSize { .. }
        | GameError::InvalidDeckContents => RuleViolation::InvalidPattern,
        GameError::PlayDoesNotBeatCurrent => RuleViolation::PlayDoesNotBeatCurrent,
    }
}

pub(super) fn duration_ceil_seconds(duration: Duration) -> u16 {
    let milliseconds = duration.as_millis();
    u16::try_from(milliseconds.div_ceil(1_000)).unwrap_or(u16::MAX)
}
