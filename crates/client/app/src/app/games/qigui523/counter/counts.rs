use leocard_protocol::QiGui523Snapshot;
use leocard_qigui523::{QiGuiCard, QiGuiRank, QiGuiSuit};
use std::collections::HashSet;

pub(super) struct CounterCell {
    pub card: QiGuiCard,
    pub remaining: u8,
}

#[derive(Default)]
pub(super) struct CardCounts {
    excluded: HashSet<QiGuiCard>,
    deck_count: u8,
}

impl CardCounts {
    pub fn update(&mut self, game: &QiGui523Snapshot, deck_count: u8) {
        self.deck_count = deck_count;
        self.excluded.clear();
        self.excluded
            .extend(game.your_hand.iter().chain(&game.played_cards).copied());
    }

    pub fn rows(&self) -> [[Option<CounterCell>; 14]; 4] {
        [
            QiGuiSuit::Spade,
            QiGuiSuit::Heart,
            QiGuiSuit::Club,
            QiGuiSuit::Diamond,
        ]
        .map(|suit| {
            let mut ranks = QiGuiRank::IN_STRENGTH_ORDER;
            ranks.reverse();
            ranks.map(|rank| {
                if rank.is_joker() && !matches!(suit, QiGuiSuit::Spade | QiGuiSuit::Club) {
                    return None;
                }
                let known = self
                    .excluded
                    .iter()
                    .filter(|card| card.rank() == rank && card.suit() == suit)
                    .count();
                Some(CounterCell {
                    card: QiGuiCard::suited(0, suit, rank),
                    remaining: self.deck_count.saturating_sub(known as u8),
                })
            })
        })
    }
}
