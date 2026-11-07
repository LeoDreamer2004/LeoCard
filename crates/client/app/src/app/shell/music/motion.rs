use crate::app::shell::{CozyModalKind, ModalAnimations, UiState, advance_modal};
use bevy::prelude::*;

#[derive(Resource, Default)]
pub(crate) struct MusicMotion {
    pub progress: f32,
    pub target_open: bool,
    pub visible: bool,
}

impl MusicMotion {
    pub fn toggle(&mut self) {
        self.target_open = !self.target_open;
        self.visible |= self.target_open;
    }

    pub fn close(&mut self) {
        self.target_open = false;
    }
}

pub(super) fn advance_music_modal(
    time: Res<Time>,
    mut motion: ResMut<MusicMotion>,
    mut ui: ResMut<UiState>,
    mut animations: ResMut<ModalAnimations>,
) {
    let open = motion.target_open;
    advance_modal(&mut motion.progress, open, time.delta_secs());
    if motion.visible && !open && motion.progress == 0.0 {
        motion.visible = false;
        ui.dirty = true;
    }
    animations.set(CozyModalKind::Music, motion.progress);
}
