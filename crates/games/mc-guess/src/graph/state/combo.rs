use super::McGuessGraphGame;
use crate::McGuessGameError;

#[derive(Clone, Copy, Debug)]
pub(super) struct Combo {
    pub anchor: usize,
    pub count: u16,
}

impl McGuessGraphGame {
    pub(super) fn available_nodes(&self) -> Vec<usize> {
        let remaining: Vec<_> = (0..self.nodes.len())
            .filter(|&index| self.entry(index).is_some() && !self.revealed[index])
            .collect();
        if let Some(combo) = self.combo {
            let neighbours: Vec<_> = self
                .neighbours(combo.anchor)
                .into_iter()
                .filter(|index| remaining.contains(index))
                .collect();
            if !neighbours.is_empty() {
                return neighbours;
            }
        }
        remaining
    }

    pub(super) fn check_combo_target(&self, index: usize) -> Result<(), McGuessGameError> {
        if self.combo.is_some() && !self.available_nodes().contains(&index) {
            return Err(McGuessGameError::ComboRequiresNeighbour);
        }
        Ok(())
    }

    pub(super) fn end_turn(&mut self) {
        self.combo = None;
        self.current = (self.current + 1) % self.players.len();
    }
}
