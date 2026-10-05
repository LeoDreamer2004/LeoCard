//! Host-only showdown equity. Future deck order never participates in the calculation.

use leocard_texas_holdem::{GameState, TexasHoldemCard, build_deck, evaluate_player_hand};
use std::{
    cmp::Ordering,
    sync::{
        Arc, Mutex,
        atomic::{AtomicU64, Ordering as AtomicOrdering},
    },
    thread,
};

#[derive(Clone, Debug, Default)]
pub(super) struct EquityCache(Arc<Mutex<CacheState>>);

#[derive(Debug, Default)]
struct CacheState {
    key: Option<EquityKey>,
    values: Option<Vec<u16>>,
    announced: bool,
    generation: Arc<AtomicU64>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct EquityKey {
    hand: u32,
    board: Vec<TexasHoldemCard>,
    players: Vec<(bool, Vec<TexasHoldemCard>)>,
}

impl EquityCache {
    pub(super) fn values(&self, game: &GameState) -> Option<Vec<u16>> {
        let key = EquityKey {
            hand: game.hand_number(),
            board: game.community().to_vec(),
            players: game
                .players()
                .iter()
                .map(|player| (player.folded(), player.hole_cards().to_vec()))
                .collect(),
        };
        let mut cache = self.0.lock().expect("equity cache lock is not poisoned");
        if cache.key.as_ref() == Some(&key) {
            return cache.values.clone();
        }
        let generation = cache.generation.fetch_add(1, AtomicOrdering::Relaxed) + 1;
        let cancellation = cache.generation.clone();
        cache.key = Some(key);
        cache.values = None;
        cache.announced = false;
        let destination = self.0.clone();
        let game = game.clone();
        thread::spawn(move || {
            let Some(values) = EquityCalculation::new(&game, &cancellation, generation).calculate()
            else {
                return;
            };
            let mut cache = destination
                .lock()
                .expect("equity cache lock is not poisoned");
            if cancellation.load(AtomicOrdering::Relaxed) == generation {
                cache.values = Some(values);
            }
        });
        None
    }

    pub(super) fn take_ready(&self) -> bool {
        let mut cache = self.0.lock().expect("equity cache lock is not poisoned");
        let ready = cache.values.is_some() && !cache.announced;
        cache.announced |= ready;
        ready
    }

    pub(super) fn clear(&self) {
        let mut cache = self.0.lock().expect("equity cache lock is not poisoned");
        if cache.key.take().is_some() {
            cache.generation.fetch_add(1, AtomicOrdering::Relaxed);
        }
        cache.values = None;
        cache.announced = false;
    }
}

struct EquityCalculation<'a> {
    game: &'a GameState,
    board: Vec<TexasHoldemCard>,
    available: Vec<TexasHoldemCard>,
    shares: Vec<u64>,
    cancellation: &'a AtomicU64,
    generation: u64,
    outcomes: usize,
}

impl<'a> EquityCalculation<'a> {
    fn new(game: &'a GameState, cancellation: &'a AtomicU64, generation: u64) -> Self {
        let available = build_deck(game.rules().short_deck)
            .into_iter()
            .filter(|card| {
                !game.community().contains(card)
                    && !game
                        .players()
                        .iter()
                        .any(|player| player.hole_cards().contains(card))
            })
            .collect();
        Self {
            game,
            board: game.community().to_vec(),
            available,
            shares: vec![0; game.players().len()],
            cancellation,
            generation,
            outcomes: 0,
        }
    }

    fn calculate(mut self) -> Option<Vec<u16>> {
        assert!(
            (3..=5).contains(&self.board.len()),
            "spectator equity starts after the flop"
        );
        self.enumerate_boards(0);
        if self.cancellation.load(AtomicOrdering::Relaxed) != self.generation {
            return None;
        }
        // 60 is divisible by every possible split-pot winner count (1..=6).
        // Integer accumulation preserves exact split shares throughout enumeration.
        let denominator = self.outcomes as u64 * 60;
        Some(
            self.shares
                .iter()
                .map(|share| ((share * 10_000 + denominator / 2) / denominator) as u16)
                .collect(),
        )
    }

    fn enumerate_boards(&mut self, start: usize) {
        if self.cancellation.load(AtomicOrdering::Relaxed) != self.generation {
            return;
        }
        if self.board.len() == 5 {
            self.record_outcome();
            return;
        }
        let missing = 5 - self.board.len();
        for index in start..=self.available.len() - missing {
            self.board.push(self.available[index]);
            self.enumerate_boards(index + 1);
            self.board.pop();
        }
    }

    fn record_outcome(&mut self) {
        let hands = self
            .game
            .players()
            .iter()
            .enumerate()
            .filter(|(_, player)| !player.folded())
            .map(|(index, player)| {
                let hand =
                    evaluate_player_hand(player.hole_cards(), &self.board, self.game.rules())
                        .expect("authoritative cards form a legal showdown hand");
                (index, hand)
            })
            .collect::<Vec<_>>();
        let best = hands
            .iter()
            .map(|(_, hand)| hand)
            .max_by(|left, right| left.cmp_with_rules(right, self.game.rules()))
            .expect("active players exist");
        let count = hands
            .iter()
            .filter(|(_, hand)| hand.cmp_with_rules(best, self.game.rules()) == Ordering::Equal)
            .count();
        for (index, hand) in &hands {
            if hand.cmp_with_rules(best, self.game.rules()) == Ordering::Equal {
                self.shares[*index] += 60 / count as u64;
            }
        }
        self.outcomes += 1;
    }
}
