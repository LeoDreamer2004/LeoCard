use std::collections::HashSet;
use std::hash::Hash;

/// Physical-card selection only. Legality, hints and modal choices stay game-owned.
pub(crate) trait HandSelection {
    type Card: Copy + Eq + Hash;

    fn selected_cards(&self) -> &HashSet<Self::Card>;

    fn selected_cards_mut(&mut self) -> &mut HashSet<Self::Card>;

    fn clear_selection(&mut self) {
        self.selected_cards_mut().clear();
    }

    fn replace_selection(&mut self, cards: impl IntoIterator<Item = Self::Card>) {
        *self.selected_cards_mut() = cards.into_iter().collect();
    }
}
