use super::FourPlayerObservation;
use crate::DoudizhuGameState;
use leocard_game_common::SearchRandom;

impl FourPlayerObservation {
    pub(super) fn sample(&self, random: &mut SearchRandom) -> DoudizhuGameState {
        let mut hands = vec![Vec::new(); 4];
        hands[usize::from(self.actor)] = self.hand.clone();
        // Visible kitty cards still belong to the landlord until publicly played.
        hands[usize::from(self.landlord)].extend_from_slice(&self.landlord_known);
        let mut pool: Vec<_> = self
            .unknown
            .iter()
            .copied()
            .filter(|card| !self.landlord_known.contains(card))
            .collect();
        for end in (1..pool.len()).rev() {
            let other = random.index(end + 1);
            pool.swap(end, other);
        }
        for (player, hand) in hands.iter_mut().enumerate() {
            let needed = self.counts[player] - hand.len();
            hand.extend(pool.drain(pool.len() - needed..));
            hand.sort_unstable();
        }
        debug_assert!(pool.is_empty());
        DoudizhuGameState::sampled_position(
            self.rules,
            hands,
            self.actor,
            self.landlord,
            self.table.clone(),
        )
    }
}
