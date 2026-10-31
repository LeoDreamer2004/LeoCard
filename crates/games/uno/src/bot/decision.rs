use super::{UnoBotAction, UnoBotObservation};
use crate::UnoPendingSwap;
use fastrand::Rng;

pub struct UnoBot;

impl UnoBot {
    pub fn choose(observation: &UnoBotObservation, rng: &mut Rng) -> Option<UnoBotAction> {
        let turn = observation.turn;
        let player = turn.current_player;
        let play = |card| UnoBotAction::Play {
            card,
            color: Self::automatic_chosen_color(observation, card),
        };
        let action = if observation.forced_skip {
            UnoBotAction::ResolveSkip
        } else if let Some(pending) = turn.pending_swap {
            match pending {
                UnoPendingSwap::SwapOneTarget { .. } => {
                    let target = observation
                        .opponents
                        .iter()
                        .max_by_key(|opponent| opponent.hand_len)?;
                    if target.hand_len == 0 {
                        return None;
                    }
                    UnoBotAction::SwapOneTarget {
                        target: target.id,
                        index: rng.usize(..target.hand_len),
                    }
                }
                UnoPendingSwap::SwapOneGive { .. } => {
                    let card = observation
                        .hand
                        .iter()
                        .copied()
                        .max_by_key(|card| card.score())?;
                    UnoBotAction::GiveSwapOneCard(card)
                }
                UnoPendingSwap::ForceTrade { .. } => {
                    let second = observation
                        .opponents
                        .iter()
                        .min_by_key(|opponent| opponent.hand_len)?
                        .id;
                    UnoBotAction::ForceTrade {
                        first: player,
                        second,
                    }
                }
                UnoPendingSwap::ChooseColor { .. } | UnoPendingSwap::ColorRoulette { .. } => {
                    UnoBotAction::ChooseColor(Self::preferred_color(
                        &observation.hand,
                        observation.flip_side,
                    ))
                }
                UnoPendingSwap::SevenSwap { .. } => {
                    let target = observation
                        .opponents
                        .iter()
                        .min_by_key(|opponent| opponent.hand_len)?
                        .id;
                    UnoBotAction::SevenSwap(target)
                }
            }
        } else if turn.current_color.is_none() {
            UnoBotAction::ChooseColor(Self::preferred_color(
                &observation.hand,
                observation.flip_side,
            ))
        } else if turn.pending_kind.is_some() {
            observation
                .playable
                .first()
                .copied()
                .map(play)
                .unwrap_or(UnoBotAction::AcceptDrawPenalty)
        } else if turn.pending_skip > 0 || turn.skipped_turns_remaining > 0 {
            if turn.skipped_turns_remaining == 0
                && let Some(&card) = observation.playable.first()
            {
                UnoBotAction::Play { card, color: None }
            } else {
                UnoBotAction::ResolveSkip
            }
        } else if let Some(card) = turn.drawn_card {
            play(card)
        } else {
            observation
                .playable
                .first()
                .copied()
                .map(play)
                .unwrap_or(UnoBotAction::Draw)
        };
        Some(action)
    }
}
