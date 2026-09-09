use super::{
    COLOR_ROULETTE_REVEAL_START_DELAY, DRAW_REVEAL_START_DELAY, PendingDrawReveal, from_core_player,
};
use leocard_protocol::UnoEvent;
use leocard_uno::{ActionOutcome, GameState, Phase, PlayedEffect, UnoCard, UnoColor};

pub(super) fn events_for_outcome(
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

pub(super) fn append_finished_event(events: &mut Vec<UnoEvent>, game: &GameState) {
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

pub(super) fn pending_draw_reveal_for_outcome(
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
