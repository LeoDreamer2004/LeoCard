use leocard_game_common::PlayerId;
use leocard_protocol::UnoViolation;
use leocard_uno::{
    UnoCard, UnoGameError, UnoGameState, UnoProfileStats, UnoRuleSet, build_deck_for_rules,
    build_flip_dark_sides, pair_flip_deck,
};

use std::collections::HashSet;

pub(super) fn shuffled_uno_deck(rules: UnoRuleSet) -> Vec<UnoCard> {
    let mut deck = if rules.is_flip() && rules.flip.random_pairing {
        let mut dark_sides = build_flip_dark_sides();
        fastrand::shuffle(&mut dark_sides);
        pair_flip_deck(dark_sides).expect("a complete FLIP dark side set pairs with the light set")
    } else {
        build_deck_for_rules(rules)
    };
    fastrand::shuffle(&mut deck);
    deck
}

pub(super) fn validate_deck(deck: &[UnoCard], rules: UnoRuleSet) -> Result<(), UnoGameError> {
    let expected_deck = build_deck_for_rules(rules);
    if deck.len() != expected_deck.len() {
        return Err(UnoGameError::InvalidDeckSize {
            expected: expected_deck.len(),
            actual: deck.len(),
        });
    }
    if rules.is_flip() {
        let mut actual_light = deck
            .iter()
            .map(|card| (card.color(), card.face()))
            .collect::<Vec<_>>();
        let mut expected_light = expected_deck
            .iter()
            .map(|card| (card.color(), card.face()))
            .collect::<Vec<_>>();
        let mut actual_dark = deck
            .iter()
            .filter_map(|card| card.opposite().map(|side| (side.color(), side.face())))
            .collect::<Vec<_>>();
        let mut expected_dark = expected_deck
            .iter()
            .filter_map(|card| card.opposite().map(|side| (side.color(), side.face())))
            .collect::<Vec<_>>();
        actual_light.sort_unstable();
        expected_light.sort_unstable();
        actual_dark.sort_unstable();
        expected_dark.sort_unstable();
        if actual_light != expected_light
            || actual_dark != expected_dark
            || deck.iter().copied().collect::<HashSet<_>>().len() != deck.len()
        {
            return Err(UnoGameError::InvalidDeckContents);
        }
        return Ok(());
    }
    let expected = expected_deck.into_iter().collect::<HashSet<_>>();
    let actual = deck.iter().copied().collect::<HashSet<_>>();
    if actual.len() != deck.len() || actual != expected {
        return Err(UnoGameError::InvalidDeckContents);
    }
    Ok(())
}

pub(super) fn resolve_public_hand_card(
    game: &UnoGameState,
    player: PlayerId,
    public: UnoCard,
) -> Result<UnoCard, UnoGameError> {
    game.player(player)
        .and_then(|state| {
            state
                .hand()
                .iter()
                .copied()
                .find(|card| card.public_face() == public.public_face())
        })
        .ok_or(UnoGameError::CardNotInHand(public))
}

pub(super) fn merge_uno_profile_stats(aggregate: &mut UnoProfileStats, current: &UnoProfileStats) {
    aggregate.max_hand_cards = aggregate.max_hand_cards.max(current.max_hand_cards);
    aggregate.max_penalty_cards = aggregate.max_penalty_cards.max(current.max_penalty_cards);
    aggregate.max_skipped_turns = aggregate.max_skipped_turns.max(current.max_skipped_turns);
    aggregate.uno_calls = aggregate.uno_calls.saturating_add(current.uno_calls);
    aggregate.uno_penalties = aggregate
        .uno_penalties
        .saturating_add(current.uno_penalties);
    aggregate.challenges = aggregate.challenges.saturating_add(current.challenges);
    aggregate.successful_challenges = aggregate
        .successful_challenges
        .saturating_add(current.successful_challenges);
    aggregate.challenges_received = aggregate
        .challenges_received
        .saturating_add(current.challenges_received);
    aggregate.successful_challenges_received = aggregate
        .successful_challenges_received
        .saturating_add(current.successful_challenges_received);
    aggregate.jump_in_opportunities = aggregate
        .jump_in_opportunities
        .saturating_add(current.jump_in_opportunities);
    aggregate.successful_jump_ins = aggregate
        .successful_jump_ins
        .saturating_add(current.successful_jump_ins);
}

pub(super) fn map_game_error(error: &UnoGameError) -> UnoViolation {
    match error {
        UnoGameError::InvalidPlayer(_) => UnoViolation::InvalidPlayer,
        UnoGameError::PlayerEliminated(_) => UnoViolation::PlayerEliminated,
        UnoGameError::NotPlayersTurn { .. } => UnoViolation::NotPlayersTurn,
        UnoGameError::GameAlreadyFinished => UnoViolation::GameAlreadyFinished,
        UnoGameError::InitialColorChoiceRequired => UnoViolation::InitialColorChoiceRequired,
        UnoGameError::InitialColorAlreadyChosen => UnoViolation::InitialColorAlreadyChosen,
        UnoGameError::CardNotInHand(_) => UnoViolation::CardNotInHand,
        UnoGameError::CardDoesNotMatch => UnoViolation::CardDoesNotMatch,
        UnoGameError::ColorRequired => UnoViolation::ColorRequired,
        UnoGameError::UnexpectedColor => UnoViolation::UnexpectedColor,
        UnoGameError::MustPlayDrawnCard(_) => UnoViolation::MustPlayDrawnCard,
        UnoGameError::MustResolveDrawPenalty => UnoViolation::MustResolveDrawPenalty,
        UnoGameError::NoDrawPenalty => UnoViolation::NoDrawPenalty,
        UnoGameError::CannotStack(_) => UnoViolation::CannotStack,
        UnoGameError::CannotChallenge => UnoViolation::CannotChallenge,
        UnoGameError::MustDrawBeforePassing => UnoViolation::MustDrawBeforePassing,
        UnoGameError::MustResolveSkip => UnoViolation::MustResolveSkip,
        UnoGameError::NoSkipToResolve => UnoViolation::NoSkipToResolve,
        UnoGameError::UnoCalloutDisabled => UnoViolation::UnoCalloutDisabled,
        UnoGameError::CannotCallUno(_) => UnoViolation::CannotCallUno,
        UnoGameError::MustPlayAfterUno => UnoViolation::MustPlayAfterUno,
        UnoGameError::CannotReportSelf => UnoViolation::CannotReportSelf,
        UnoGameError::PlayerNotReportable(_) => UnoViolation::PlayerNotReportable,
        UnoGameError::CannotPlayTogether => UnoViolation::CannotPlayTogether,
        UnoGameError::CannotJumpIn => UnoViolation::CannotJumpIn,
        UnoGameError::MustResolveSwapEffect => UnoViolation::MustResolveSwapEffect,
        UnoGameError::NoSwapEffect => UnoViolation::NoSwapEffect,
        UnoGameError::InvalidSwapTargets => UnoViolation::InvalidSwapTargets,
        UnoGameError::DrawPileExhausted => UnoViolation::DrawPileExhausted,
        UnoGameError::InvalidRules(_)
        | UnoGameError::InvalidDeckSize { .. }
        | UnoGameError::InvalidDeckContents => UnoViolation::CardDoesNotMatch,
    }
}
