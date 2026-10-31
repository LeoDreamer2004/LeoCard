use super::{COLOR_ROULETTE_REVEAL_START_DELAY, DRAW_REVEAL_START_DELAY, PendingDrawReveal};
use leocard_protocol::UnoEvent;
use leocard_uno::{UnoActionOutcome, UnoCard, UnoColor, UnoGameState, UnoPhase, UnoPlayedEffect};

pub(super) fn events_for_outcome(
    outcome: &UnoActionOutcome,
    played: &[(UnoCard, Option<UnoColor>)],
) -> Vec<UnoEvent> {
    let mut events = played
        .iter()
        .copied()
        .enumerate()
        .map(|(index, (card, chosen_color))| match outcome {
            UnoActionOutcome::Played { player, .. } => UnoEvent::CardPlayed {
                player: *player,
                card: card.public_face(),
                chosen_color,
                play_index: index as u8,
                play_count: played.len() as u8,
            },
            _ => unreachable!("only play actions carry played-card metadata"),
        })
        .collect::<Vec<_>>();
    match outcome {
        UnoActionOutcome::ColorChosen { player, color } => events.push(UnoEvent::ColorChosen {
            player: *player,
            color: *color,
        }),
        UnoActionOutcome::DrewCards { player, cards, .. } if !cards.is_empty() => {
            events.push(UnoEvent::CardsDrawn {
                player: *player,
                count: cards.len() as u16,
                penalty: false,
                card_backs: public_card_backs(cards),
            })
        }
        UnoActionOutcome::PenaltyDrawn { player, cards, .. } if !cards.is_empty() => {
            events.push(UnoEvent::CardsDrawn {
                player: *player,
                count: cards.len() as u16,
                penalty: true,
                card_backs: public_card_backs(cards),
            });
        }
        UnoActionOutcome::ChallengeResolved {
            challenger,
            offender,
            result,
            penalized,
            cards,
            ..
        } => events.push(UnoEvent::ChallengeResolved {
            challenger: *challenger,
            offender: *offender,
            result: *result,
            penalized: *penalized,
            count: cards.len() as u16,
            card_backs: public_card_backs(cards),
        }),
        UnoActionOutcome::UnoCalled { player } => {
            events.push(UnoEvent::UnoCalled { player: *player })
        }
        UnoActionOutcome::UnoReported {
            reporter,
            target,
            cards,
        } => events.push(UnoEvent::UnoReported {
            reporter: *reporter,
            target: *target,
            card_backs: public_card_backs(cards),
        }),
        UnoActionOutcome::SkipResolved {
            player,
            cards,
            remaining,
            ..
        } => events.push(UnoEvent::SkipResolved {
            player: *player,
            remaining: *remaining,
            drew_card: !cards.is_empty(),
            card_back: cards.first().and_then(|card| card.opposite_public_face()),
        }),
        UnoActionOutcome::SwapOneCardTaken { player, target } => {
            events.push(UnoEvent::SwapOneCardTaken {
                player: *player,
                target: *target,
            });
        }
        UnoActionOutcome::SwapOneCompleted { player, target, .. } => {
            events.push(UnoEvent::SwapOneCompleted {
                player: *player,
                target: *target,
            });
        }
        UnoActionOutcome::HandsTraded {
            player,
            first,
            second,
        } => events.push(UnoEvent::HandsTraded {
            player: *player,
            first: *first,
            second: *second,
        }),
        UnoActionOutcome::Played { player, effect, .. } => match effect {
            Some(UnoPlayedEffect::HandRefreshed { count }) => {
                events.push(UnoEvent::HandRefreshed {
                    player: *player,
                    count: *count,
                });
            }
            Some(UnoPlayedEffect::HandsPassed { direction }) => {
                events.push(UnoEvent::HandsPassed {
                    player: *player,
                    direction: *direction,
                });
            }
            Some(UnoPlayedEffect::DrawReflected {
                player: target,
                cards,
            }) => {
                events.push(UnoEvent::DrawPenaltyReflected {
                    player: *player,
                    target: *target,
                    count: cards.len() as u16,
                    card_backs: public_card_backs(cards),
                });
            }
            Some(UnoPlayedEffect::StackNumberRevealed { cards, value }) => {
                events.push(UnoEvent::StackNumberRevealed {
                    player: *player,
                    cards: cards.clone(),
                    value: *value,
                });
            }
            Some(UnoPlayedEffect::CardsDiscarded { cards }) => {
                events.push(UnoEvent::CardsDiscarded {
                    player: *player,
                    cards: cards.iter().map(|card| card.public_face()).collect(),
                });
            }
            Some(UnoPlayedEffect::Flipped { side }) => {
                events.push(UnoEvent::Flipped { side: *side });
            }
            None => {}
        },
        UnoActionOutcome::ColorRouletteResolved {
            player,
            color,
            cards,
            ..
        } => events.push(UnoEvent::ColorRouletteResolved {
            player: *player,
            color: *color,
            count: cards.len() as u16,
            card_backs: public_card_backs(cards),
        }),
        UnoActionOutcome::DrewCards { .. }
        | UnoActionOutcome::PenaltyDrawn { .. }
        | UnoActionOutcome::PassedAfterDraw { .. } => {}
    }
    events
}

fn public_card_backs(cards: &[UnoCard]) -> Vec<UnoCard> {
    cards
        .iter()
        .filter_map(|card| card.opposite_public_face())
        .collect()
}

pub(super) fn append_finished_event(events: &mut Vec<UnoEvent>, game: &UnoGameState) {
    if events
        .iter()
        .any(|event| matches!(event, UnoEvent::GameFinished { .. }))
    {
        return;
    }
    if let UnoPhase::Finished(result) = game.phase() {
        events.push(UnoEvent::GameFinished {
            winner: result.winner,
        });
    }
}

pub(super) fn pending_draw_reveal_for_outcome(
    outcome: &UnoActionOutcome,
    game: &UnoGameState,
) -> Option<PendingDrawReveal> {
    let (player, cards, remaining) = match outcome {
        UnoActionOutcome::DrewCards { player, cards, .. } if cards.len() > 1 => {
            (*player, cards, DRAW_REVEAL_START_DELAY)
        }
        UnoActionOutcome::ColorRouletteResolved { player, cards, .. } if cards.len() > 1 => {
            (*player, cards, COLOR_ROULETTE_REVEAL_START_DELAY)
        }
        _ => return None,
    };
    if game.player(player).is_none_or(|state| state.eliminated()) {
        return None;
    }
    Some(PendingDrawReveal {
        player,
        cards: cards.clone(),
        revealed: 0,
        remaining,
    })
}
