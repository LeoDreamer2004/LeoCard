use leocard_protocol::{ShengjiEvent, ShengjiPublicPlay, ShengjiViolation};
use leocard_shengji::{
    ShengjiBidError, ShengjiCard, ShengjiComponent, ShengjiFollowError, ShengjiGameError,
    ShengjiHandResult, ShengjiPlayError, ShengjiProfileStats, ShengjiRuleSet, ShengjiSuit,
    ShengjiTrickRecord, ShengjiTrump, build_deck_for,
};

use std::collections::HashSet;

pub(super) fn record_shengji_component(
    stats: &mut ShengjiProfileStats,
    component: &ShengjiComponent,
) {
    let index = match component {
        ShengjiComponent::Single { .. }
        | ShengjiComponent::Pair { .. }
        | ShengjiComponent::Triple { .. } => return,
        ShengjiComponent::Tractor { pair_count, .. } => {
            stats.longest_tractor = stats.longest_tractor.max(u16::from(*pair_count));
            0
        }
        ShengjiComponent::Titanic { triple_count, .. } => {
            stats.longest_titanic = stats.longest_titanic.max(u16::from(*triple_count));
            1
        }
        ShengjiComponent::Quad { .. } => 2,
        ShengjiComponent::Spaceship { quad_count, .. } => {
            stats.longest_spaceship = stats.longest_spaceship.max(u16::from(*quad_count));
            3
        }
    };
    stats.play_category_counts[index] = stats.play_category_counts[index].saturating_add(1);
}

pub(super) fn merge_shengji_profile_stats(
    aggregate: &mut ShengjiProfileStats,
    current: &ShengjiProfileStats,
) {
    aggregate.declaration_games = aggregate
        .declaration_games
        .saturating_add(current.declaration_games);
    aggregate.counter_games = aggregate
        .counter_games
        .saturating_add(current.counter_games);
    aggregate.plays = aggregate.plays.saturating_add(current.plays);
    aggregate.winning_plays = aggregate
        .winning_plays
        .saturating_add(current.winning_plays);
    aggregate.crossing_games = aggregate
        .crossing_games
        .saturating_add(current.crossing_games);
    for (aggregate, current) in aggregate
        .play_category_counts
        .iter_mut()
        .zip(current.play_category_counts)
    {
        *aggregate = aggregate.saturating_add(current);
    }
    aggregate.longest_tractor = aggregate.longest_tractor.max(current.longest_tractor);
    aggregate.longest_titanic = aggregate.longest_titanic.max(current.longest_titanic);
    aggregate.longest_spaceship = aggregate.longest_spaceship.max(current.longest_spaceship);
    aggregate.longest_throw = aggregate.longest_throw.max(current.longest_throw);
}

pub(super) fn completed_trick_events(
    trick: &ShengjiTrickRecord,
    collecting_score: u32,
) -> Vec<ShengjiEvent> {
    let last = trick
        .plays
        .last()
        .map(|(player, play)| ShengjiEvent::CardsPlayed {
            play: ShengjiPublicPlay {
                player: *player,
                play: play.clone(),
                throw_penalty: 0,
            },
            is_lead: false,
        });
    last.into_iter()
        .chain([ShengjiEvent::TrickFinished {
            winner: trick.winner,
            points: trick.points,
            collecting_score,
        }])
        .collect()
}

pub(super) fn pre_kitty_collecting_score(result: &ShengjiHandResult) -> u32 {
    i64::from(result.trick_points)
        .saturating_add(i64::from(result.penalty_adjustment))
        .max(0) as u32
}

pub(super) fn finished_reference_point_magnitude(result: &ShengjiHandResult) -> i16 {
    if result.promoted_team == result.dealer_team {
        i16::from(result.promoted_steps) * 2
    } else {
        i16::from(result.promoted_steps.saturating_add(1)) * 2
    }
}

pub(super) fn game_violation(error: ShengjiGameError) -> ShengjiViolation {
    match error {
        ShengjiGameError::InvalidPlayer(_) => ShengjiViolation::InvalidPlayer,
        ShengjiGameError::WrongPhase
        | ShengjiGameError::RedealRequired
        | ShengjiGameError::Rules(_) => ShengjiViolation::WrongPhase,
        ShengjiGameError::NotDealer => ShengjiViolation::NotDealer,
        ShengjiGameError::WrongBuryCount { expected, actual } => ShengjiViolation::WrongBuryCount {
            expected: expected.min(usize::from(u8::MAX)) as u8,
            actual: actual.min(usize::from(u8::MAX)) as u8,
        },
        ShengjiGameError::CardsNotOwned => ShengjiViolation::CardsNotOwned,
        ShengjiGameError::CrossingNotEligible => ShengjiViolation::CrossingNotEligible,
        ShengjiGameError::CrossingAlreadyDecided => ShengjiViolation::CrossingAlreadyDecided,
        ShengjiGameError::WrongCrossingCount { expected, actual } => {
            ShengjiViolation::WrongCrossingCount {
                expected: expected.min(usize::from(u8::MAX)) as u8,
                actual: actual.min(usize::from(u8::MAX)) as u8,
            }
        }
        ShengjiGameError::CrossingMustIncludeAllTrumps => {
            ShengjiViolation::CrossingMustIncludeAllTrumps
        }
        ShengjiGameError::CrossingReturnNotRequired => ShengjiViolation::CrossingReturnNotRequired,
        ShengjiGameError::CrossingAlreadyReturned => ShengjiViolation::CrossingAlreadyReturned,
        ShengjiGameError::NotBottomCopyPlayer => ShengjiViolation::NotBottomCopyPlayer,
        ShengjiGameError::NotPlayersTurn { .. } => ShengjiViolation::NotPlayersTurn,
        ShengjiGameError::Bid(error) => match error {
            ShengjiBidError::InvalidPlayer(_) => ShengjiViolation::InvalidPlayer,
            ShengjiBidError::Closed
            | ShengjiBidError::NoDeclaration
            | ShengjiBidError::InvalidCards => ShengjiViolation::InvalidDeclaration,
            ShengjiBidError::CardsNotOwned => ShengjiViolation::DeclarationCardsNotOwned,
            ShengjiBidError::CounterRequiresPair => ShengjiViolation::CounterRequiresPair,
            ShengjiBidError::NotStronger => ShengjiViolation::CounterNotStronger,
            ShengjiBidError::ProtectedSuit => {
                ShengjiViolation::ProtectedSuitCanOnlyBeCounteredByNoTrump
            }
            ShengjiBidError::JokerRequired => ShengjiViolation::DeclarationRequiresJoker,
            ShengjiBidError::NoTrumpCannotOpen => ShengjiViolation::NoTrumpCannotOpen,
        },
        ShengjiGameError::Play(error) => play_violation(error),
        ShengjiGameError::Follow(error) => match error {
            ShengjiFollowError::Play(error) => play_violation(error),
            ShengjiFollowError::WrongCardCount { expected, actual } => {
                ShengjiViolation::WrongCardCount {
                    expected: expected.min(usize::from(u8::MAX)) as u8,
                    actual: actual.min(usize::from(u8::MAX)) as u8,
                }
            }
            ShengjiFollowError::MustFollowCategory { .. } => ShengjiViolation::MustFollowCategory,
            ShengjiFollowError::MustFollowStructure => ShengjiViolation::MustFollowStructure,
        },
        ShengjiGameError::InvalidDeckSize { .. } | ShengjiGameError::InvalidDeckContents => {
            ShengjiViolation::WrongPhase
        }
    }
}

fn play_violation(error: ShengjiPlayError) -> ShengjiViolation {
    match error {
        ShengjiPlayError::Empty => ShengjiViolation::MustLeadWithCards,
        ShengjiPlayError::DuplicatePhysicalCard | ShengjiPlayError::CardsNotOwned => {
            ShengjiViolation::CardsNotOwned
        }
        ShengjiPlayError::MixedCategory => ShengjiViolation::InvalidPattern,
        ShengjiPlayError::ThrowDisabled => ShengjiViolation::ThrowDisabled,
    }
}

pub(super) fn card_sort_key(card: ShengjiCard, trump: ShengjiTrump) -> (bool, u8, u8, u8) {
    (
        trump.is_trump(card),
        trump.strength(card),
        card.suit().map_or(4, ShengjiSuit::bid_strength),
        card.deck(),
    )
}

pub(super) fn validate_deck(
    deck: &[ShengjiCard],
    rules: ShengjiRuleSet,
) -> Result<(), ShengjiGameError> {
    let expected = build_deck_for(rules.deck_count);
    if deck.len() != expected.len() {
        return Err(ShengjiGameError::InvalidDeckSize {
            expected: expected.len(),
            actual: deck.len(),
        });
    }
    let expected = expected.into_iter().collect::<HashSet<_>>();
    let actual = deck.iter().copied().collect::<HashSet<_>>();
    if actual.len() != deck.len() || actual != expected {
        return Err(ShengjiGameError::InvalidDeckContents);
    }
    Ok(())
}

pub(super) fn shuffled_deck(rules: ShengjiRuleSet) -> Vec<ShengjiCard> {
    let mut deck = build_deck_for(rules.deck_count);
    fastrand::shuffle(&mut deck);
    deck
}
