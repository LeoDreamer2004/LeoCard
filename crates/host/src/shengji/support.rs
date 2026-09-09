use crate::HostError;
use leocard_protocol::{
    PlayerId, ShengjiBottomFlipMatchView, ShengjiBottomFlipRevealView, ShengjiEvent,
    ShengjiProfileStats, ShengjiPublicPlay, ShengjiViolation,
};
use leocard_shengji::{
    BidError, BottomFlipReveal, Component, FollowError, GameError, HandResult, PlayError,
    ShengjiCard, ShengjiPlayerId, ShengjiRuleSet, ShengjiSuit, ShengjiTrump, TrickRecord,
    build_deck_for,
};
use std::collections::HashSet;

pub(super) fn record_shengji_component(stats: &mut ShengjiProfileStats, component: &Component) {
    let index = match component {
        Component::Single { .. } | Component::Pair { .. } | Component::Triple { .. } => return,
        Component::Tractor { pair_count, .. } => {
            stats.longest_tractor = stats.longest_tractor.max(u16::from(*pair_count));
            0
        }
        Component::Titanic { triple_count, .. } => {
            stats.longest_titanic = stats.longest_titanic.max(u16::from(*triple_count));
            1
        }
        Component::Quad { .. } => 2,
        Component::Spaceship { quad_count, .. } => {
            stats.longest_space_fortress = stats.longest_space_fortress.max(u16::from(*quad_count));
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
    aggregate.longest_space_fortress = aggregate
        .longest_space_fortress
        .max(current.longest_space_fortress);
    aggregate.longest_throw = aggregate.longest_throw.max(current.longest_throw);
}

pub(super) fn completed_trick_events(
    trick: &TrickRecord,
    collecting_score: u32,
) -> Vec<ShengjiEvent> {
    let last = trick
        .plays
        .last()
        .map(|(player, play)| ShengjiEvent::CardsPlayed {
            play: ShengjiPublicPlay {
                player: from_core_player(*player),
                play: play.clone(),
                throw_penalty: 0,
            },
            is_lead: false,
        });
    last.into_iter()
        .chain([ShengjiEvent::TrickFinished {
            winner: from_core_player(trick.winner),
            points: trick.points,
            collecting_score,
        }])
        .collect()
}

pub(super) fn bottom_flip_reveal_view(reveal: &BottomFlipReveal) -> ShengjiBottomFlipRevealView {
    ShengjiBottomFlipRevealView {
        card: reveal.card,
        matches: reveal
            .matches
            .iter()
            .map(|matched| ShengjiBottomFlipMatchView {
                player: from_core_player(matched.player),
                cards: matched.cards.clone(),
            })
            .collect(),
        dealer: reveal.dealer.map(from_core_player),
    }
}

pub(super) fn pre_kitty_collecting_score(result: &HandResult) -> u32 {
    i64::from(result.trick_points)
        .saturating_add(i64::from(result.penalty_adjustment))
        .max(0) as u32
}

pub(super) fn finished_reference_point_magnitude(result: &HandResult) -> i16 {
    if result.promoted_team == result.dealer_team {
        i16::from(result.promoted_steps) * 2
    } else {
        i16::from(result.promoted_steps.saturating_add(1)) * 2
    }
}

pub(super) fn game_violation(error: GameError) -> ShengjiViolation {
    match error {
        GameError::InvalidPlayer(_) => ShengjiViolation::InvalidPlayer,
        GameError::WrongPhase | GameError::RedealRequired | GameError::Rules(_) => {
            ShengjiViolation::WrongPhase
        }
        GameError::NotDealer => ShengjiViolation::NotDealer,
        GameError::WrongBuryCount { expected, actual } => ShengjiViolation::WrongBuryCount {
            expected: expected.min(usize::from(u8::MAX)) as u8,
            actual: actual.min(usize::from(u8::MAX)) as u8,
        },
        GameError::CardsNotOwned => ShengjiViolation::CardsNotOwned,
        GameError::CrossingNotEligible => ShengjiViolation::CrossingNotEligible,
        GameError::CrossingAlreadyDecided => ShengjiViolation::CrossingAlreadyDecided,
        GameError::WrongCrossingCount { expected, actual } => {
            ShengjiViolation::WrongCrossingCount {
                expected: expected.min(usize::from(u8::MAX)) as u8,
                actual: actual.min(usize::from(u8::MAX)) as u8,
            }
        }
        GameError::CrossingMustIncludeAllTrumps => ShengjiViolation::CrossingMustIncludeAllTrumps,
        GameError::CrossingReturnNotRequired => ShengjiViolation::CrossingReturnNotRequired,
        GameError::CrossingAlreadyReturned => ShengjiViolation::CrossingAlreadyReturned,
        GameError::NotBottomCopyPlayer => ShengjiViolation::NotBottomCopyPlayer,
        GameError::NotPlayersTurn { .. } => ShengjiViolation::NotPlayersTurn,
        GameError::Bid(error) => match error {
            BidError::InvalidPlayer(_) => ShengjiViolation::InvalidPlayer,
            BidError::Closed | BidError::NoDeclaration | BidError::InvalidCards => {
                ShengjiViolation::InvalidDeclaration
            }
            BidError::CardsNotOwned => ShengjiViolation::DeclarationCardsNotOwned,
            BidError::CounterRequiresPair => ShengjiViolation::CounterRequiresPair,
            BidError::NotStronger => ShengjiViolation::CounterNotStronger,
            BidError::ProtectedSuit => ShengjiViolation::ProtectedSuitCanOnlyBeCounteredByNoTrump,
            BidError::JokerRequired => ShengjiViolation::DeclarationRequiresJoker,
            BidError::NoTrumpCannotOpen => ShengjiViolation::NoTrumpCannotOpen,
        },
        GameError::Play(error) => play_violation(error),
        GameError::Follow(error) => match error {
            FollowError::Play(error) => play_violation(error),
            FollowError::WrongCardCount { expected, actual } => ShengjiViolation::WrongCardCount {
                expected: expected.min(usize::from(u8::MAX)) as u8,
                actual: actual.min(usize::from(u8::MAX)) as u8,
            },
            FollowError::MustFollowCategory { .. } => ShengjiViolation::MustFollowCategory,
            FollowError::MustFollowStructure => ShengjiViolation::MustFollowStructure,
        },
        GameError::InvalidDeckSize { .. } | GameError::InvalidDeckContents => {
            ShengjiViolation::WrongPhase
        }
    }
}

fn play_violation(error: PlayError) -> ShengjiViolation {
    match error {
        PlayError::Empty => ShengjiViolation::MustLeadWithCards,
        PlayError::DuplicatePhysicalCard | PlayError::CardsNotOwned => {
            ShengjiViolation::CardsNotOwned
        }
        PlayError::MixedCategory => ShengjiViolation::InvalidPattern,
        PlayError::ThrowDisabled => ShengjiViolation::ThrowDisabled,
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

pub(super) fn validate_deck(deck: &[ShengjiCard], rules: ShengjiRuleSet) -> Result<(), HostError> {
    let expected = build_deck_for(rules.deck_count);
    if deck.len() != expected.len() {
        return Err(HostError::InvalidDeckSize {
            expected: expected.len(),
            actual: deck.len(),
        });
    }
    let expected = expected.into_iter().collect::<HashSet<_>>();
    let actual = deck.iter().copied().collect::<HashSet<_>>();
    if actual.len() != deck.len() || actual != expected {
        return Err(HostError::InvalidDeckContents);
    }
    Ok(())
}

pub(super) fn shuffled_deck(rules: ShengjiRuleSet) -> Vec<ShengjiCard> {
    let mut deck = build_deck_for(rules.deck_count);
    fastrand::shuffle(&mut deck);
    deck
}

pub(super) const fn to_core_player(player: PlayerId) -> ShengjiPlayerId {
    ShengjiPlayerId(player.0)
}

pub(super) const fn from_core_player(player: ShengjiPlayerId) -> PlayerId {
    PlayerId(player.0)
}
