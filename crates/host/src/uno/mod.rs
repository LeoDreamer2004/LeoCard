use std::collections::HashSet;
use std::time::Duration;

use leocard_protocol::{
    ClientCommand, ClientMessage, GameCommand, GameEvent, GameKind, GameRules, GameSnapshot,
    GameViolation, LobbySnapshot, MatchId, PlayerId, PlayerInteraction, PlayerInteractionKind,
    PlayerReferenceChange, RejectReason, RequestId, Revision, RoomId, ServerEvent, UnoCommand,
    UnoEvent, UnoPendingSwapView, UnoPhaseView, UnoPlayerResult, UnoPlayerState, UnoProfileStats,
    UnoRevealedHand, UnoSnapshot, UnoViolation,
};
use leocard_uno::{
    ActionOutcome, GameError, GameState, PendingSwap, Phase, PlayedEffect, UnoCard,
    UnoChallengeResult, UnoColor, UnoFace, UnoFlipSide, UnoPlayerId, UnoRuleSet,
    build_deck_for_rules, build_flip_dark_sides, pair_flip_deck,
};
#[cfg(test)]
use leocard_uno::{build_deck, build_no_mercy_deck};

use crate::{
    AUTO_PLAY_DELAY, AutoPlayDelayState, ConnectionId, Delivery, HostError, RoomSession,
    new_match_id,
};

mod automation;
mod commands;
mod lifecycle;
mod room;
mod settlement;
mod snapshot;

#[cfg(test)]
mod tests;

const DRAW_REVEAL_START_DELAY: Duration = Duration::from_millis(780);
const COLOR_ROULETTE_REVEAL_START_DELAY: Duration = Duration::from_millis(1_000);
const DRAW_REVEAL_INTERVAL: Duration = Duration::from_millis(180);

#[derive(Clone, Debug)]
struct PendingDrawReveal {
    player: PlayerId,
    cards: Vec<UnoCard>,
    revealed: usize,
    remaining: Duration,
}

#[derive(Clone, Debug)]
pub struct UnoSession {
    room: RoomSession,
    rules: UnoRuleSet,
    shuffled_deck: Option<Vec<UnoCard>>,
    game: Option<GameState>,
    match_id: Option<MatchId>,
    match_profile_stats: Vec<UnoProfileStats>,
    finished_reference_changes: Option<Vec<PlayerReferenceChange>>,
    auto_play_delay: Option<AutoPlayDelayState>,
    pending_draw_reveal: Option<PendingDrawReveal>,
}

fn events_for_outcome(
    outcome: &ActionOutcome,
    played: &[(UnoCard, Option<UnoColor>)],
) -> Vec<UnoEvent> {
    let mut events = played
        .iter()
        .copied()
        .enumerate()
        .map(|(index, (card, chosen_color))| match outcome {
            ActionOutcome::Played { player, .. } => UnoEvent::CardPlayed {
                player: from_core_player(*player),
                card: card.public_face(),
                chosen_color,
                play_index: index as u8,
                play_count: played.len() as u8,
            },
            _ => unreachable!("only play actions carry played-card metadata"),
        })
        .collect::<Vec<_>>();
    match outcome {
        ActionOutcome::ColorChosen { player, color } => events.push(UnoEvent::ColorChosen {
            player: from_core_player(*player),
            color: *color,
        }),
        ActionOutcome::DrewCards { player, cards, .. } => events.push(UnoEvent::CardsDrawn {
            player: from_core_player(*player),
            count: cards.len() as u16,
            penalty: false,
            card_backs: public_card_backs(cards),
        }),
        ActionOutcome::PenaltyDrawn { player, cards, .. } => {
            events.push(UnoEvent::CardsDrawn {
                player: from_core_player(*player),
                count: cards.len() as u16,
                penalty: true,
                card_backs: public_card_backs(cards),
            });
        }
        ActionOutcome::ChallengeResolved {
            challenger,
            offender,
            result,
            penalized,
            cards,
            ..
        } => events.push(UnoEvent::ChallengeResolved {
            challenger: from_core_player(*challenger),
            offender: from_core_player(*offender),
            result: *result,
            penalized: from_core_player(*penalized),
            count: cards.len() as u16,
            card_backs: public_card_backs(cards),
        }),
        ActionOutcome::UnoCalled { player } => events.push(UnoEvent::UnoCalled {
            player: from_core_player(*player),
        }),
        ActionOutcome::UnoReported {
            reporter,
            target,
            cards,
        } => events.push(UnoEvent::UnoReported {
            reporter: from_core_player(*reporter),
            target: from_core_player(*target),
            card_backs: public_card_backs(cards),
        }),
        ActionOutcome::SkipResolved {
            player,
            cards,
            remaining,
            ..
        } => events.push(UnoEvent::SkipResolved {
            player: from_core_player(*player),
            remaining: *remaining,
            drew_card: !cards.is_empty(),
            card_back: cards.first().and_then(|card| card.opposite_public_face()),
        }),
        ActionOutcome::SwapOneCardTaken { player, target } => {
            events.push(UnoEvent::SwapOneCardTaken {
                player: from_core_player(*player),
                target: from_core_player(*target),
            });
        }
        ActionOutcome::SwapOneCompleted { player, target, .. } => {
            events.push(UnoEvent::SwapOneCompleted {
                player: from_core_player(*player),
                target: from_core_player(*target),
            });
        }
        ActionOutcome::HandsTraded {
            player,
            first,
            second,
        } => events.push(UnoEvent::HandsTraded {
            player: from_core_player(*player),
            first: from_core_player(*first),
            second: from_core_player(*second),
        }),
        ActionOutcome::Played { player, effect, .. } => match effect {
            Some(PlayedEffect::HandRefreshed { count }) => {
                events.push(UnoEvent::HandRefreshed {
                    player: from_core_player(*player),
                    count: *count,
                });
            }
            Some(PlayedEffect::HandsPassed { direction }) => {
                events.push(UnoEvent::HandsPassed {
                    player: from_core_player(*player),
                    direction: *direction,
                });
            }
            Some(PlayedEffect::DrawReflected {
                player: target,
                cards,
            }) => {
                events.push(UnoEvent::DrawPenaltyReflected {
                    player: from_core_player(*player),
                    target: from_core_player(*target),
                    count: cards.len() as u16,
                    card_backs: public_card_backs(cards),
                });
            }
            Some(PlayedEffect::StackNumberRevealed { cards, value }) => {
                events.push(UnoEvent::StackNumberRevealed {
                    player: from_core_player(*player),
                    cards: cards.clone(),
                    value: *value,
                });
            }
            Some(PlayedEffect::CardsDiscarded { cards }) => {
                events.push(UnoEvent::CardsDiscarded {
                    player: from_core_player(*player),
                    cards: cards.iter().map(|card| card.public_face()).collect(),
                });
            }
            Some(PlayedEffect::Flipped { side }) => {
                events.push(UnoEvent::Flipped { side: *side });
            }
            None => {}
        },
        ActionOutcome::ColorRouletteResolved {
            player,
            color,
            cards,
            ..
        } => events.push(UnoEvent::ColorRouletteResolved {
            player: from_core_player(*player),
            color: *color,
            count: cards.len() as u16,
            card_backs: public_card_backs(cards),
        }),
        ActionOutcome::PassedAfterDraw { .. } => {}
    }
    events
}

fn public_card_backs(cards: &[UnoCard]) -> Vec<UnoCard> {
    cards
        .iter()
        .filter_map(|card| card.opposite_public_face())
        .collect()
}

fn append_finished_event(events: &mut Vec<UnoEvent>, game: &GameState) {
    if events
        .iter()
        .any(|event| matches!(event, UnoEvent::GameFinished { .. }))
    {
        return;
    }
    if let Phase::Finished(result) = game.phase() {
        events.push(UnoEvent::GameFinished {
            winner: from_core_player(result.winner),
        });
    }
}

fn pending_draw_reveal_for_outcome(
    outcome: &ActionOutcome,
    game: &GameState,
) -> Option<PendingDrawReveal> {
    let (player, cards, remaining) = match outcome {
        ActionOutcome::DrewCards { player, cards, .. } if cards.len() > 1 => {
            (*player, cards, DRAW_REVEAL_START_DELAY)
        }
        ActionOutcome::ColorRouletteResolved { player, cards, .. } if cards.len() > 1 => {
            (*player, cards, COLOR_ROULETTE_REVEAL_START_DELAY)
        }
        _ => return None,
    };
    if game.player(player).is_none_or(|state| state.eliminated()) {
        return None;
    }
    Some(PendingDrawReveal {
        player: from_core_player(player),
        cards: cards.clone(),
        revealed: 0,
        remaining,
    })
}

fn preferred_color(game: &GameState, player: UnoPlayerId) -> UnoColor {
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

fn automatic_chosen_color(
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

fn shuffled_uno_deck(rules: UnoRuleSet) -> Vec<UnoCard> {
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

fn validate_deck(deck: &[UnoCard], rules: UnoRuleSet) -> Result<(), HostError> {
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

fn to_core_player(player: PlayerId) -> UnoPlayerId {
    UnoPlayerId(usize::from(player.0))
}

fn from_core_player(player: UnoPlayerId) -> PlayerId {
    PlayerId(u8::try_from(player.0).expect("UNO supports at most six players"))
}

fn resolve_public_hand_card(
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

fn merge_uno_profile_stats(aggregate: &mut UnoProfileStats, current: &UnoProfileStats) {
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

fn map_game_error(error: &GameError) -> UnoViolation {
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
