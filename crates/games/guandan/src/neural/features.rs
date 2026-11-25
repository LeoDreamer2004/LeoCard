use super::actions::action_id;
use crate::{
    GuandanAction, GuandanCard, GuandanGameState, GuandanPhase, GuandanPlayKind, GuandanSuit,
    hint_options,
};
use leocard_game_common::PlayerId;
use std::collections::VecDeque;

#[derive(Clone, Debug, Default)]
pub struct GuandanModelHistory {
    played: [Vec<GuandanCard>; 4],
    last: [Vec<GuandanCard>; 4],
    round: [Vec<GuandanCard>; 4],
    history: VecDeque<Vec<GuandanCard>>,
}

impl GuandanModelHistory {
    pub fn record(&mut self, player: PlayerId, action: &GuandanAction, next: Option<PlayerId>) {
        let cards = match action {
            GuandanAction::Play(cards) => cards.clone(),
            GuandanAction::Pass => Vec::new(),
            _ => return,
        };
        self.played[player.index()].extend_from_slice(&cards);
        self.last[player.index()] = cards.clone();
        self.round[player.index()] = cards;
        if next.is_some_and(|next| next.0 <= player.0) {
            self.history.extend(self.round.iter().cloned());
            while self.history.len() > 20 {
                self.history.pop_front();
            }
            self.round = Default::default();
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct GuandanModelObservation {
    pub(super) features: Vec<f32>,
    pub(super) actions: Vec<(usize, GuandanAction)>,
}

impl GuandanModelObservation {
    pub fn from_game(game: &GuandanGameState, history: &GuandanModelHistory) -> Option<Self> {
        if !matches!(game.phase(), GuandanPhase::Playing) {
            return None;
        }
        let player = game.current_player()?;
        let hand = game.player(player)?.hand();
        let level = game.rules().starting_level;
        let mut features = vec![0.0; 3049];
        features[..108].copy_from_slice(&encode_cards(hand));
        // Preserve the published checkpoint's absolute-seat layout, including its
        // overlapping fourth-seat channels. Changing the layout needs retraining.
        for (i, state) in game.players().iter().enumerate() {
            if state.id() != player {
                features[108 + i] = state.hand().len().min(26) as f32 / 26.0;
            }
        }
        for i in 0..4 {
            features[111 + i * 108..111 + (i + 1) * 108]
                .copy_from_slice(&encode_cards(&history.last[i]));
        }
        for i in 0..4 {
            if i != player.index() {
                features[543 + i * 108..543 + (i + 1) * 108]
                    .copy_from_slice(&encode_cards(&history.played[i]));
            }
        }
        // The source encoder maps numeric J/Q/K/A levels to its default slot.
        let level_index = if level <= 10 {
            usize::from(level - 2)
        } else {
            0
        };
        features[867 + level_index] = 1.0;
        let padding = 20 - history.history.len();
        for (i, action) in history.history.iter().enumerate() {
            let start = 880 + (padding + i) * 108;
            features[start..start + 108].copy_from_slice(&encode_cards(action));
        }
        for start in [3040, 3043, 3046] {
            features[start] = 1.0;
        }
        let options = hint_options(hand, level, game.winning_play());
        // Nine/ten-card bombs are outside the released checkpoint's vocabulary.
        if options
            .iter()
            .any(|play| matches!(play.kind, GuandanPlayKind::Bomb(9 | 10)))
        {
            return None;
        }
        let mut actions: Vec<_> = options
            .into_iter()
            .filter_map(|play| {
                action_id(&play, level).map(|id| (id, GuandanAction::Play(play.cards)))
            })
            .collect();
        if game.winning_play().is_some() {
            actions.push((0, GuandanAction::Pass));
        }
        (!actions.is_empty()).then_some(Self { features, actions })
    }
}

fn encode_cards(cards: &[GuandanCard]) -> [f32; 108] {
    let mut counts = [0u8; 54];
    for card in cards {
        let index = match card.suit {
            GuandanSuit::Spades => usize::from(card.rank - 2),
            GuandanSuit::Hearts => 13 + usize::from(card.rank - 2),
            GuandanSuit::Clubs => 26 + usize::from(card.rank - 2),
            GuandanSuit::Diamonds => 39 + usize::from(card.rank - 2),
            GuandanSuit::Joker => 52 + usize::from(card.rank == 17),
        };
        counts[index] += 1;
    }
    let mut encoded = [0.0; 108];
    for (index, count) in counts.iter().enumerate() {
        for copy in 0..usize::from((*count).min(2)) {
            let position = if index < 52 {
                copy * 52 + index
            } else {
                104 + copy * 2 + index - 52
            };
            encoded[position] = 1.0;
        }
    }
    encoded
}
