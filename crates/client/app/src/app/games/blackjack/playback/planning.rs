use super::state::{
    BlackjackCardSlot, BlackjackCardTiming, BlackjackPlayback, DEALER_FLIP_DURATION,
};
use crate::app::presentation::CARD_DEAL_INTERVAL;
use leocard_blackjack::{BlackjackCard, Phase};
use leocard_protocol::BlackjackSnapshot;
use std::slice;

impl BlackjackPlayback {
    pub(super) fn observe(&mut self, game: &BlackjackSnapshot) -> Vec<f32> {
        if self
            .observed
            .as_ref()
            .is_none_or(|before| (before.match_id, before.round) != (game.match_id, game.round))
        {
            *self = Self::default();
        }
        if self.observed.as_ref() == Some(game) {
            return Vec::new();
        }
        let previous = self.observed.take();
        let mut sounds = Vec::new();
        let initial = previous.as_ref().is_none_or(|game| {
            game.players
                .iter()
                .all(|view| view.hands.iter().all(|hand| hand.cards.is_empty()))
        }) && game
            .players
            .iter()
            .any(|view| view.hands.iter().any(|hand| !hand.cards.is_empty()));
        if initial {
            self.initial_deal(game, &mut sounds);
        } else if let Some(before) = &previous {
            self.player_draws(before, game, &mut sounds);
        }
        if game.phase == Phase::Complete
            && previous
                .as_ref()
                .is_none_or(|before| before.phase != Phase::Complete)
        {
            self.reveal_dealer(game, &mut sounds);
        }
        self.observed = Some(game.clone());
        sounds
    }

    fn schedule_deal(&mut self, slot: BlackjackCardSlot, at: f32, sounds: &mut Vec<f32>) {
        let timing = BlackjackCardTiming {
            deal_at: Some(at),
            flip_at: None,
        };
        self.busy_until = self.busy_until.max(timing.finishes_at(slot));
        self.cards.insert(slot, timing);
        sounds.push((at - self.elapsed).max(0.0));
    }

    fn initial_deal(&mut self, game: &BlackjackSnapshot, sounds: &mut Vec<f32>) {
        let mut players: Vec<_> = game.players.iter().collect();
        players.sort_by_key(|view| view.player.seat.map(|seat| seat.0));
        let mut at = self.elapsed;
        for index in 0..2 {
            for player in &players {
                for (hand_index, hand) in player.hands.iter().enumerate() {
                    if hand.cards.get(index).is_some() {
                        self.schedule_deal(
                            BlackjackCardSlot::Hand(player.player.id, hand_index, index),
                            at,
                            sounds,
                        );
                        at += CARD_DEAL_INTERVAL;
                    }
                }
            }
            if game.dealer.get(index).is_some() || (index == 1 && game.dealer_hidden) {
                self.schedule_deal(BlackjackCardSlot::Dealer(index), at, sounds);
                at += CARD_DEAL_INTERVAL;
            }
        }
        for player in players {
            for (hand_index, hand) in player.hands.iter().enumerate() {
                for index in 2..hand.cards.len() {
                    self.schedule_deal(
                        BlackjackCardSlot::Hand(player.player.id, hand_index, index),
                        at,
                        sounds,
                    );
                    at += CARD_DEAL_INTERVAL;
                }
            }
        }
    }

    fn player_draws(
        &mut self,
        before: &BlackjackSnapshot,
        game: &BlackjackSnapshot,
        sounds: &mut Vec<f32>,
    ) {
        let mut at = self.busy_until.max(self.elapsed);
        for player in &game.players {
            let Some(old) = before
                .players
                .iter()
                .find(|view| view.player.id == player.player.id)
            else {
                continue;
            };
            let mut old_hands: Vec<&[BlackjackCard]> =
                old.hands.iter().map(|hand| hand.cards.as_slice()).collect();
            if before.phase == Phase::Playing
                && before.current_player == Some(player.player.id)
                && player.hands.len() == old_hands.len() + 1
                && let Some(index) = before.current_hand.map(usize::from)
                && let Some([first, second]) = old_hands.get(index).copied()
            {
                old_hands.splice(
                    index..=index,
                    [slice::from_ref(first), slice::from_ref(second)],
                );
                self.cards.retain(|slot, _| !matches!(slot, BlackjackCardSlot::Hand(id, _, _) if *id == player.player.id));
            }
            for (hand_index, hand) in player.hands.iter().enumerate() {
                let old_cards = old_hands.get(hand_index).copied().unwrap_or_default();
                for (index, card) in hand.cards.iter().enumerate() {
                    if old_cards.get(index) != Some(card) {
                        self.schedule_deal(
                            BlackjackCardSlot::Hand(player.player.id, hand_index, index),
                            at,
                            sounds,
                        );
                        at += 0.12;
                    }
                }
            }
        }
    }

    fn reveal_dealer(&mut self, game: &BlackjackSnapshot, sounds: &mut Vec<f32>) {
        let mut at = self.busy_until.max(self.elapsed) + 0.4;
        if game.dealer.len() > 1 {
            let timing = self.cards.entry(BlackjackCardSlot::Dealer(1)).or_default();
            timing.flip_at = Some(at);
            self.busy_until = self.busy_until.max(at + DEALER_FLIP_DURATION);
            sounds.push((at - self.elapsed).max(0.0));
            at += DEALER_FLIP_DURATION + 0.4;
        }
        for index in 2..game.dealer.len() {
            self.schedule_deal(BlackjackCardSlot::Dealer(index), at, sounds);
            at = self.busy_until + 0.4;
        }
    }
}
