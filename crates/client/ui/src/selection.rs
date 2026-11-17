use bevy::prelude::*;
use std::collections::HashSet;
use std::hash::Hash;

#[derive(Resource, Default)]
pub struct CardDragSelection {
    pub active: bool,
    pub anchor: usize,
    pub current: usize,
    pub select: bool,
    visited: Option<HashSet<usize>>,
}

impl CardDragSelection {
    pub fn begin(&mut self, index: usize, select: bool) {
        self.active = true;
        self.anchor = index;
        self.current = index;
        self.select = select;
        self.visited = None;
    }

    pub fn begin_path(&mut self, index: usize, select: bool) {
        self.begin(index, select);
        self.visited = Some(HashSet::from([index]));
    }

    pub fn visit(&mut self, index: usize) {
        self.current = index;
        if let Some(visited) = &mut self.visited {
            visited.insert(index);
        }
    }

    pub fn contains(&self, index: usize) -> bool {
        self.active
            && self.visited.as_ref().map_or_else(
                || (self.anchor.min(self.current)..=self.anchor.max(self.current)).contains(&index),
                |visited| visited.contains(&index),
            )
    }

    pub fn finish<C: Copy + Eq + Hash>(
        &mut self,
        selected: &mut HashSet<C>,
        cards: impl Iterator<Item = (usize, C)>,
    ) -> bool {
        let mut changed = false;
        for (index, card) in cards {
            if self.contains(index) {
                changed |= if self.select {
                    selected.insert(card)
                } else {
                    selected.remove(&card)
                };
            }
        }
        self.active = false;
        changed
    }
}
