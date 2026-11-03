use super::actions::legal_actions;
use crate::{
    DoudizhuAction, DoudizhuCard, DoudizhuGameState, DoudizhuMode, DoudizhuPhase, DoudizhuPlayKind,
    build_deck, classify,
};
use leocard_game_common::PlayerId;

/// Only the actor's hand, public counts and public action history cross into inference.
#[derive(Clone, Debug, PartialEq)]
pub struct DouzeroObservation {
    pub(crate) role: usize,
    pub(crate) features: Vec<f32>,
    pub(crate) history: Vec<f32>,
    pub(crate) actions: Vec<DoudizhuAction>,
}

impl DouzeroObservation {
    pub fn from_game(
        game: &DoudizhuGameState,
        history: &[(PlayerId, DoudizhuAction)],
    ) -> Option<Self> {
        if game.rules.mode != DoudizhuMode::Three || game.phase != DoudizhuPhase::Playing {
            return None;
        }
        let landlord = game.landlord?.0 as usize;
        let own = game.current.0 as usize;
        let down = (landlord + 1) % 3;
        let up = (landlord + 2) % 3;
        let hand = game.hand(game.current)?;
        let counts = game.hand_lengths();
        let unknown: Vec<_> = build_deck(DoudizhuMode::Three)
            .into_iter()
            .filter(|card| !hand.contains(card) && !game.played.contains(card))
            .collect();
        let mut played: [Vec<DoudizhuCard>; 3] = Default::default();
        let mut last: [Vec<DoudizhuCard>; 3] = Default::default();
        let mut sequence = Vec::new();
        let mut bombs = 0;
        for (player, action) in history {
            let cards = match action {
                DoudizhuAction::Play(cards) => cards.as_slice(),
                DoudizhuAction::Pass => &[],
                DoudizhuAction::Bid(_) => continue,
            };
            played[usize::from(*player)].extend_from_slice(cards);
            last[usize::from(*player)] = cards.to_vec();
            if classify(cards, game.rules).is_some_and(|play| {
                matches!(play.kind, DoudizhuPlayKind::Bomb | DoudizhuPlayKind::Rocket)
            }) {
                bombs += 1;
            }
            sequence.push(cards);
        }
        let last_move = sequence
            .last()
            .copied()
            .filter(|cards| !cards.is_empty())
            .or_else(|| sequence.len().checked_sub(2).map(|index| sequence[index]))
            .unwrap_or(&[]);
        let mut features = Vec::new();
        features.extend(encode_cards(hand));
        features.extend(encode_cards(&unknown));
        let role = if own == landlord {
            0
        } else if own == down {
            1
        } else {
            2
        };
        if role == 0 {
            features.extend(encode_cards(last_move));
            features.extend(encode_cards(&played[up]));
            features.extend(encode_cards(&played[down]));
            append_count(&mut features, counts[up] as usize, 17);
            append_count(&mut features, counts[down] as usize, 17);
        } else {
            let teammate = if own == down { up } else { down };
            features.extend(encode_cards(&played[landlord]));
            features.extend(encode_cards(&played[teammate]));
            features.extend(encode_cards(last_move));
            features.extend(encode_cards(&last[landlord]));
            features.extend(encode_cards(&last[teammate]));
            append_count(&mut features, counts[landlord] as usize, 20);
            append_count(&mut features, counts[teammate] as usize, 17);
        }
        for index in 0..15 {
            features.push(f32::from(index == bombs));
        }
        let mut encoded_history = vec![0.0; 810];
        let length = sequence.len().min(15);
        for (index, cards) in sequence[sequence.len() - length..].iter().enumerate() {
            let start = (15 - length + index) * 54;
            encoded_history[start..start + 54].copy_from_slice(&encode_cards(cards));
        }
        Some(Self {
            role,
            features,
            history: encoded_history,
            actions: legal_actions(hand, game.table.as_ref().map(|(_, play)| play), game.rules),
        })
    }
}

pub(crate) fn encode_cards(cards: &[DoudizhuCard]) -> [f32; 54] {
    let mut counts = [0; 15];
    for card in cards {
        counts[card.rank() as usize] += 1;
    }
    let mut encoded = [0.0; 54];
    for rank in 0..13 {
        for copy in 0..counts[rank].min(4) {
            encoded[rank * 4 + copy] = 1.0;
        }
    }
    encoded[52] = f32::from(counts[13] > 0);
    encoded[53] = f32::from(counts[14] > 0);
    encoded
}

fn append_count(features: &mut Vec<f32>, count: usize, maximum: usize) {
    features.extend((1..=maximum).map(|index| f32::from(index == count)));
}
