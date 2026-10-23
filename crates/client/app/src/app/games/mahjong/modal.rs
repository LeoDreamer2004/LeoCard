use super::MahjongUiState;
use crate::app::shell::{CozyModalKind, ModalAnimations, advance_modal};
use bevy::prelude::*;

pub(super) fn advance_mahjong_modal(
    time: Res<Time>,
    mut state: ResMut<MahjongUiState>,
    mut animations: ResMut<ModalAnimations>,
) {
    let open = state.fan_guide_open;
    advance_modal(&mut state.fan_guide_progress, open, time.delta_secs());
    animations.set(CozyModalKind::MahjongFanGuide, state.fan_guide_progress);
}
