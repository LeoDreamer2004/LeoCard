use crate::app::presentation::{CARD_DEAL_DURATION, CARD_FLIP_DURATION};
use bevy::prelude::*;
use leocard_blackjack::Phase;
use leocard_protocol::{BlackjackPlayerView, BlackjackSnapshot, PlayerId};
use std::collections::HashMap;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub(crate) enum BlackjackCardSlot {
    Dealer(usize),
    Hand(PlayerId, usize, usize),
}

#[derive(Component, Clone, Copy, Debug, Default, PartialEq)]
pub(crate) struct BlackjackCardTiming {
    pub deal_at: Option<f32>,
    pub flip_at: Option<f32>,
}

impl BlackjackCardTiming {
    pub(super) fn finishes_at(self) -> f32 {
        self.flip_at.map_or_else(
            || self.deal_at.map_or(0.0, |at| at + CARD_DEAL_DURATION),
            |at| at + CARD_FLIP_DURATION,
        )
    }
}

#[derive(Resource, Default)]
pub(crate) struct BlackjackPlayback {
    pub(super) observed: Option<BlackjackSnapshot>,
    pub(super) elapsed: f32,
    pub(super) busy_until: f32,
    pub(super) cards: HashMap<BlackjackCardSlot, BlackjackCardTiming>,
}

impl BlackjackPlayback {
    pub(crate) fn busy(&self) -> bool {
        self.elapsed < self.busy_until
    }

    pub(in super::super) fn elapsed(&self) -> f32 {
        self.elapsed
    }

    pub(in super::super) fn timing(&self, slot: BlackjackCardSlot) -> BlackjackCardTiming {
        self.cards.get(&slot).copied().unwrap_or_default()
    }

    pub(in super::super) fn hand_ready(&self, player: PlayerId, hand: usize) -> bool {
        self.cards.iter().all(|(slot, timing)| {
            !matches!(slot, BlackjackCardSlot::Hand(id, index, _) if *id == player && *index == hand)
                || timing.finishes_at() <= self.elapsed
        })
    }

    pub(in super::super) fn displayed_stack(&self, view: &BlackjackPlayerView) -> u32 {
        if self.busy()
            && self
                .observed
                .as_ref()
                .is_some_and(|game| game.phase == Phase::Complete)
        {
            view.round_start_stack
                .saturating_sub(view.hands.iter().map(|hand| hand.wager).sum())
        } else {
            view.stack
        }
    }
}
