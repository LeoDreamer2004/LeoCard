use leocard_protocol::QiGui523Violation;
use leocard_qigui523::{
    QiGui523ProfileStats, QiGuiCard, QiGuiGameError, QiGuiPlayError, build_deck,
};

use std::collections::HashSet;
use std::time::Duration;

pub(super) fn validate_deck(deck_count: u8, deck: &[QiGuiCard]) -> Result<(), QiGuiGameError> {
    let expected_deck = build_deck(deck_count);
    if deck.len() != expected_deck.len() {
        return Err(QiGuiGameError::InvalidDeckSize {
            expected: expected_deck.len(),
            actual: deck.len(),
        });
    }
    let expected: HashSet<_> = expected_deck.into_iter().collect();
    let actual: HashSet<_> = deck.iter().copied().collect();
    if actual.len() != deck.len() || actual != expected {
        return Err(QiGuiGameError::InvalidDeckContents);
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

pub(super) fn map_game_error(error: &QiGuiGameError) -> QiGui523Violation {
    match error {
        QiGuiGameError::InvalidPlayer(_) => QiGui523Violation::InvalidPlayer,
        QiGuiGameError::NotPlayersTurn { .. } => QiGui523Violation::NotPlayersTurn,
        QiGuiGameError::GameAlreadyFinished => QiGui523Violation::GameAlreadyFinished,
        QiGuiGameError::MustLeadWithCards => QiGui523Violation::MustLeadWithCards,
        QiGuiGameError::CardNotInHand(_) => QiGui523Violation::CardNotInHand,
        QiGuiGameError::InvalidPlay(
            QiGuiPlayError::Empty
            | QiGuiPlayError::DuplicatePhysicalCard(_)
            | QiGuiPlayError::CardOutsideConfiguredDeck(_)
            | QiGuiPlayError::InvalidPattern,
        )
        | QiGuiGameError::InvalidRules(_)
        | QiGuiGameError::InvalidDeckSize { .. }
        | QiGuiGameError::InvalidDeckContents => QiGui523Violation::InvalidPattern,
        QiGuiGameError::PlayDoesNotBeatCurrent => QiGui523Violation::PlayDoesNotBeatCurrent,
    }
}

pub(super) fn duration_ceil_seconds(duration: Duration) -> u16 {
    let milliseconds = duration.as_millis();
    u16::try_from(milliseconds.div_ceil(1_000)).unwrap_or(u16::MAX)
}
