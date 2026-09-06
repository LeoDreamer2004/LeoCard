use super::*;
use leocard_protocol::{PlayerId, UnoProfileStats, UnoViolation};
use leocard_uno::{
    GameError, GameState, UnoCard, UnoColor, UnoFace, UnoFlipSide, UnoPlayerId, UnoRuleSet,
    build_deck_for_rules, build_flip_dark_sides, pair_flip_deck,
};

pub(super) fn preferred_color(game: &GameState, player: UnoPlayerId) -> UnoColor {
    let colors = match game.flip_side() {
        Some(UnoFlipSide::Dark) => UnoColor::DARK,
        Some(UnoFlipSide::Light) | None => UnoColor::LIGHT,
    };
    let mut counts = [0_u8; 4];
    if let Some(state) = game.player(player) {
        for color in state.hand().iter().filter_map(|card| card.color()) {
            if let Some(index) = colors.iter().position(|candidate| *candidate == color) {
                counts[index] = counts[index].saturating_add(1);
            }
        }
    }
    colors
        .into_iter()
        .enumerate()
        .max_by_key(|(index, _)| (counts[*index], std::cmp::Reverse(*index)))
        .map(|(_, color)| color)
        .unwrap_or(UnoColor::Red)
}

pub(super) fn automatic_chosen_color(
    game: &GameState,
    player: UnoPlayerId,
    card: UnoCard,
) -> Option<UnoColor> {
    matches!(
        card.face(),
        UnoFace::Wild
            | UnoFace::DarkWild
            | UnoFace::WildDrawFour
            | UnoFace::WildDrawTwo
            | UnoFace::WildDrawColor
            | UnoFace::WildPowerReverse
            | UnoFace::WildNoU
            | UnoFace::WildStackThree
            | UnoFace::WildStackNumber
            | UnoFace::WildReverseDrawFour
            | UnoFace::WildDrawSix
            | UnoFace::WildDrawTen
    )
    .then(|| preferred_color(game, player))
}

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

pub(super) fn validate_deck(deck: &[UnoCard], rules: UnoRuleSet) -> Result<(), HostError> {
    let expected_deck = build_deck_for_rules(rules);
    if deck.len() != expected_deck.len() {
        return Err(HostError::InvalidDeckSize {
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
            return Err(HostError::InvalidDeckContents);
        }
        return Ok(());
    }
    let expected = expected_deck.into_iter().collect::<HashSet<_>>();
    let actual = deck.iter().copied().collect::<HashSet<_>>();
    if actual.len() != deck.len() || actual != expected {
        return Err(HostError::InvalidDeckContents);
    }
    Ok(())
}

pub(super) fn to_core_player(player: PlayerId) -> UnoPlayerId {
    UnoPlayerId(usize::from(player.0))
}

pub(super) fn from_core_player(player: UnoPlayerId) -> PlayerId {
    PlayerId(u8::try_from(player.0).expect("UNO supports at most six players"))
}

pub(super) fn resolve_public_hand_card(
    game: &GameState,
    player: UnoPlayerId,
    public: UnoCard,
) -> Result<UnoCard, GameError> {
    game.player(player)
        .and_then(|state| {
            state
                .hand()
                .iter()
                .copied()
                .find(|card| card.public_face() == public.public_face())
        })
        .ok_or(GameError::CardNotInHand(public))
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

pub(super) fn map_game_error(error: &GameError) -> UnoViolation {
    match error {
        GameError::InvalidPlayer(_) => UnoViolation::InvalidPlayer,
        GameError::PlayerEliminated(_) => UnoViolation::PlayerEliminated,
        GameError::NotPlayersTurn { .. } => UnoViolation::NotPlayersTurn,
        GameError::GameAlreadyFinished => UnoViolation::GameAlreadyFinished,
        GameError::InitialColorChoiceRequired => UnoViolation::InitialColorChoiceRequired,
        GameError::InitialColorAlreadyChosen => UnoViolation::InitialColorAlreadyChosen,
        GameError::CardNotInHand(_) => UnoViolation::CardNotInHand,
        GameError::CardDoesNotMatch => UnoViolation::CardDoesNotMatch,
        GameError::ColorRequired => UnoViolation::ColorRequired,
        GameError::UnexpectedColor => UnoViolation::UnexpectedColor,
        GameError::MustPlayDrawnCard(_) => UnoViolation::MustPlayDrawnCard,
        GameError::MustResolveDrawPenalty => UnoViolation::MustResolveDrawPenalty,
        GameError::NoDrawPenalty => UnoViolation::NoDrawPenalty,
        GameError::CannotStack(_) => UnoViolation::CannotStack,
        GameError::CannotChallenge => UnoViolation::CannotChallenge,
        GameError::MustDrawBeforePassing => UnoViolation::MustDrawBeforePassing,
        GameError::MustResolveSkip => UnoViolation::MustResolveSkip,
        GameError::NoSkipToResolve => UnoViolation::NoSkipToResolve,
        GameError::UnoCalloutDisabled => UnoViolation::UnoCalloutDisabled,
        GameError::CannotCallUno(_) => UnoViolation::CannotCallUno,
        GameError::MustPlayAfterUno => UnoViolation::MustPlayAfterUno,
        GameError::CannotReportSelf => UnoViolation::CannotReportSelf,
        GameError::PlayerNotReportable(_) => UnoViolation::PlayerNotReportable,
        GameError::CannotPlayTogether => UnoViolation::CannotPlayTogether,
        GameError::CannotJumpIn => UnoViolation::CannotJumpIn,
        GameError::MustResolveSwapEffect => UnoViolation::MustResolveSwapEffect,
        GameError::NoSwapEffect => UnoViolation::NoSwapEffect,
        GameError::InvalidSwapTargets => UnoViolation::InvalidSwapTargets,
        GameError::DrawPileExhausted => UnoViolation::DrawPileExhausted,
        GameError::InvalidRules(_)
        | GameError::InvalidDeckSize { .. }
        | GameError::InvalidDeckContents => UnoViolation::CardDoesNotMatch,
    }
}
