use crate::app::presentation::CARD_DEAL_DURATION;
use bevy::prelude::*;
use leocard_blackjack::BlackjackPhase;
use leocard_game_common::PlayerId;
use leocard_protocol::{BlackjackPlayerView, BlackjackSnapshot};
use std::collections::HashMap;

// Playback completion must use the same durations as the rendered animations.
pub(in super::super) const DEALER_FLIP_DURATION: f32 = 0.7;
pub(in super::super) const SETTLEMENT_CHIP_DURATION: f32 = 0.56;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub(crate) enum BlackjackCardSlot {
    Dealer(usize),
    Hand(PlayerId, usize, usize),
}

impl BlackjackCardSlot {
    pub(in super::super) fn deal_duration(self) -> f32 {
        match self {
            Self::Dealer(index) if index >= 2 => 0.55,
            _ => CARD_DEAL_DURATION,
        }
    }
}

#[derive(Component, Clone, Copy, Debug, Default, PartialEq)]
pub(crate) struct BlackjackCardTiming {
    pub deal_at: Option<f32>,
    pub flip_at: Option<f32>,
}

impl BlackjackCardTiming {
    pub(super) fn finishes_at(self, slot: BlackjackCardSlot) -> f32 {
        self.flip_at.map_or_else(
            || self.deal_at.map_or(0.0, |at| at + slot.deal_duration()),
            |at| at + DEALER_FLIP_DURATION,
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
        let settlement = if self
            .observed
            .as_ref()
            .is_some_and(|game| game.phase == BlackjackPhase::Complete)
        {
            SETTLEMENT_CHIP_DURATION
        } else {
            0.0
        };
        self.elapsed < self.busy_until + settlement
    }

    pub(in super::super) fn settlement_started(&self) -> bool {
        self.elapsed >= self.busy_until
            && self
                .observed
                .as_ref()
                .is_some_and(|game| game.phase == BlackjackPhase::Complete)
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
                || timing.finishes_at(*slot) <= self.elapsed
        })
    }

    pub(in super::super) fn displayed_stack(&self, view: &BlackjackPlayerView) -> u32 {
        if self.busy()
            && self
                .observed
                .as_ref()
                .is_some_and(|game| game.phase == BlackjackPhase::Complete)
        {
            view.round_start_stack
                .saturating_sub(view.hands.iter().map(|hand| hand.wager).sum())
        } else {
            view.stack
        }
    }
}
