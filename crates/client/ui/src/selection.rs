use bevy::prelude::*;

#[derive(Resource, Default)]
pub struct CardDragSelection {
    pub active: bool,
    pub anchor: usize,
    pub current: usize,
    pub select: bool,
}

impl CardDragSelection {
    pub fn contains(&self, index: usize) -> bool {
        self.active
            && (self.anchor.min(self.current)..=self.anchor.max(self.current)).contains(&index)
    }
}
