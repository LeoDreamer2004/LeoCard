use super::actions::dispatch_music_actions;
use super::motion::{MusicMotion, advance_music_modal};
use super::playback::update_playback;
use super::state::MusicState;
use super::systems::poll_music;
use crate::app::runtime::ClientUpdateSet;
use crate::app::shell::{ModalAnimationSet, UiActionSet};
use bevy::prelude::*;
use leocard_ui::drag_draggable_window;

pub(crate) struct MusicPlugin;

impl Plugin for MusicPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<MusicState>()
            .init_resource::<MusicMotion>()
            .add_systems(
                Update,
                drag_draggable_window::<MusicState>.in_set(ClientUpdateSet::Input),
            )
            .add_systems(
                Update,
                advance_music_modal.in_set(ModalAnimationSet::Progress),
            )
            .add_systems(Update, dispatch_music_actions.in_set(UiActionSet))
            .add_systems(
                Update,
                (poll_music, update_playback)
                    .chain()
                    .in_set(ClientUpdateSet::Sync),
            );
    }
}
