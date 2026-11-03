use crate::{
    DoudizhuCard, DoudizhuGameState, DoudizhuMode, DoudizhuPhase, DoudizhuPlay, DoudizhuRuleSet,
    build_deck,
};
use leocard_game_common::PlayerId;

/// A four-player robot's legal information boundary. No opponent hand is retained.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FourPlayerObservation {
    pub(super) rules: DoudizhuRuleSet,
    pub(super) actor: PlayerId,
    pub(super) landlord: PlayerId,
    pub(super) hand: Vec<DoudizhuCard>,
    pub(super) counts: [usize; 4],
    pub(super) table: Option<(PlayerId, DoudizhuPlay)>,
    pub(super) unknown: Vec<DoudizhuCard>,
    pub(super) landlord_known: Vec<DoudizhuCard>,
}

impl FourPlayerObservation {
    pub fn from_game(game: &DoudizhuGameState) -> Option<Self> {
        if game.rules.mode != DoudizhuMode::Four || game.phase != DoudizhuPhase::Playing {
            return None;
        }
        let landlord = game.landlord?;
        let mut hand = game.hand(game.current)?.to_vec();
        hand.sort_unstable();
        let unknown: Vec<_> = build_deck(DoudizhuMode::Four)
            .into_iter()
            .filter(|card| !hand.contains(card) && !game.played.contains(card))
            .collect();
        let landlord_known = game
            .public_kitty()
            .iter()
            .copied()
            .filter(|card| unknown.contains(card))
            .collect();
        Some(Self {
            rules: game.rules,
            actor: game.current,
            landlord,
            hand,
            counts: game
                .hand_lengths()
                .into_iter()
                .map(usize::from)
                .collect::<Vec<_>>()
                .try_into()
                .ok()?,
            table: game.table.clone(),
            unknown,
            landlord_known,
        })
    }

    pub(super) fn seed(&self) -> u64 {
        let mut seed = u64::from(self.actor.0) + 4 * u64::from(self.landlord.0);
        for card in self
            .hand
            .iter()
            .chain(&self.unknown)
            .chain(&self.landlord_known)
        {
            seed = seed
                .wrapping_mul(1099511628211)
                .wrapping_add(u64::from(card.0) + 1);
        }
        for count in self.counts {
            seed = seed.wrapping_mul(37).wrapping_add(count as u64);
        }
        seed
    }
}
