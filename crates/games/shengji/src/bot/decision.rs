use super::ShengjiBotObservation;
use super::burial::bury;
use super::candidates::hand_value;
use super::search::choose_play;
use crate::{FiveTrumpCrossingStage, Phase, ShengjiCard, ShengjiTrump};
use leocard_game_common::SearchBudget;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ShengjiBotAction {
    Play(Vec<ShengjiCard>),
    Bury(Vec<ShengjiCard>),
    Copy(Option<Vec<ShengjiCard>>),
    Cross(Option<Vec<ShengjiCard>>),
    Return(Vec<ShengjiCard>),
}

pub struct ShengjiBot;

impl ShengjiBot {
    pub fn choose(observation: &ShengjiBotObservation, budget: &SearchBudget) -> ShengjiBotAction {
        match observation.phase {
            Phase::Burying | Phase::BottomCopyBurying => ShengjiBotAction::Bury(bury(
                &observation.hand,
                observation.rules.kitty_size(),
                observation.trump,
                budget,
            )),
            Phase::BottomCopying => {
                let current = hand_value(&observation.hand, observation.trump);
                let mut best = None;
                let mut value = current + 4.0;
                for cards in &observation.counter_options {
                    let suit = cards.iter().find_map(|card| card.suit());
                    let trump = ShengjiTrump::new(observation.trump.level, suit)
                        .expect("level")
                        .with_constant_trump(observation.rules.constant_trump);
                    let strength = hand_value(&observation.hand, trump);
                    if strength > value {
                        value = strength;
                        best = Some(cards.clone());
                    }
                }
                ShengjiBotAction::Copy(best)
            }
            Phase::FiveTrumpCrossing
                if observation.crossing == Some(FiveTrumpCrossingStage::Returning) =>
            {
                ShengjiBotAction::Return(bury(&observation.hand, 5, observation.trump, budget))
            }
            Phase::FiveTrumpCrossing => {
                let cards: Vec<_> = observation
                    .hand
                    .iter()
                    .copied()
                    .filter(|card| observation.trump.is_trump(*card))
                    .collect();
                ShengjiBotAction::Cross((cards.len() <= 5 && !cards.is_empty()).then_some(cards))
            }
            Phase::Playing => ShengjiBotAction::Play(choose_play(observation, budget)),
            _ => observation.fallback(),
        }
    }
}
