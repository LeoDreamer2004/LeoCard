use super::ShengjiBotObservation;
use super::burial::bury;
use super::candidates::hand_value;
use super::search::choose_play;
use crate::{ShengjiCard, ShengjiFiveTrumpCrossingStage, ShengjiPhase, ShengjiTrump};

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
            ShengjiPhase::Burying | ShengjiPhase::BottomCopyBurying => {
                ShengjiBotAction::Bury(bury(
                    &observation.hand,
                    observation.rules.kitty_size(),
                    observation.trump,
                    budget,
                ))
            }
            ShengjiPhase::BottomCopying => {
                let mut best = None;
                let mut value = f64::NEG_INFINITY;
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
            ShengjiPhase::FiveTrumpCrossing
                if observation.crossing == Some(ShengjiFiveTrumpCrossingStage::Returning) =>
            {
                ShengjiBotAction::Return(bury(&observation.hand, 5, observation.trump, budget))
            }
            ShengjiPhase::FiveTrumpCrossing => {
                let cards: Vec<_> = observation
                    .hand
                    .iter()
                    .copied()
                    .filter(|card| observation.trump.is_trump(*card))
                    .collect();
                ShengjiBotAction::Cross((cards.len() <= 5 && !cards.is_empty()).then_some(cards))
            }
            ShengjiPhase::Playing => ShengjiBotAction::Play(choose_play(observation, budget)),
            _ => observation.fallback(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{ShengjiRank, ShengjiRuleSet, ShengjiSuit, ShengjiTeamId};
    use leocard_game_common::PlayerId;
    use std::time::Duration;

    #[test]
    fn copying_is_mandatory_even_with_weaker_trumps_or_an_expired_budget() {
        let cards = vec![ShengjiCard::small_joker(0), ShengjiCard::small_joker(1)];
        let mut hand = cards.clone();
        hand.extend(
            ShengjiRank::LEVELS.map(|rank| ShengjiCard::suited(0, ShengjiSuit::Heart, rank)),
        );
        let observation = ShengjiBotObservation {
            player: PlayerId(1),
            hand,
            trump: ShengjiTrump::new(ShengjiRank::Two, Some(ShengjiSuit::Heart)).unwrap(),
            rules: ShengjiRuleSet {
                bottom_copy: true,
                ..ShengjiRuleSet::default()
            },
            phase: ShengjiPhase::BottomCopying,
            trick: None,
            unseen: Vec::new(),
            exposed: Default::default(),
            lengths: vec![0; 4],
            voids: vec![[false; 5]; 4],
            bottom: None,
            counter_options: vec![cards.clone()],
            crossing: None,
            collecting_team: ShengjiTeamId(1),
            collecting_score: 0,
        };
        let expected = ShengjiBotAction::Copy(Some(cards));
        assert_eq!(
            ShengjiBot::choose(&observation, &SearchBudget::new(Duration::ZERO)),
            expected
        );
        assert_eq!(observation.fallback(), expected);
    }
}
