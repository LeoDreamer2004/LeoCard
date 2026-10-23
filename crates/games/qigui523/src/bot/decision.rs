use super::{QiGui523Bot, QiGui523BotObservation, QiGui523BotRequest};
use crate::QiGuiCard;

impl QiGui523Bot {
    /// Choose a complete turn, including a lead; `None` means pass.
    pub fn choose_action(observation: &QiGui523BotObservation) -> Option<Vec<QiGuiCard>> {
        if let Some(current_play) = &observation.current_play {
            Self::new()
                .choose(QiGui523BotRequest {
                    hand: &observation.hand,
                    current_play,
                    played_cards: &observation.played_cards,
                    rules: &observation.rules,
                })
                .map(|play| play.cards().to_vec())
        } else {
            observation
                .hand
                .iter()
                .copied()
                .min_by_key(|card| (card.rank().strength(), card.suit().strength(), card.deck()))
                .map(|card| vec![card])
        }
    }
}
