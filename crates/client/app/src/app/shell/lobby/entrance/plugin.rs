use super::animation::{animate_confetti, animate_heading, animate_parties, sync_spotlight};
use super::material::EntranceVeil;
use super::state::LobbyEntranceState;
use super::view::sync_entrance_overlay;
use crate::app::runtime::{ClientUpdateSet, GameProjectionPlugin};
use crate::app::shell::UiActionSet;
use bevy::input_focus::{InputFocus, InputFocusSystems};
use bevy::prelude::*;
use bevy::ui::UiSystems;

pub(in crate::app::shell::lobby) struct LobbyEntrancePlugin;

impl Plugin for LobbyEntrancePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            GameProjectionPlugin::<LobbyEntranceState>::default(),
            UiMaterialPlugin::<EntranceVeil>::default(),
        ))
        .configure_sets(
            Update,
            (
                ClientUpdateSet::Input.run_if(interaction_available),
                UiActionSet.run_if(interaction_available),
            ),
        )
        .add_systems(
            PreUpdate,
            freeze_text_focus.before(InputFocusSystems::Dispatch),
        )
        .add_systems(Update, advance_entrance.in_set(ClientUpdateSet::Sync))
        .add_systems(
            Update,
            (
                sync_entrance_overlay,
                animate_heading,
                animate_parties,
                animate_confetti,
            )
                .chain()
                .in_set(ClientUpdateSet::Animate),
        )
        .add_systems(PostUpdate, sync_spotlight.after(UiSystems::PostLayout));
    }
}

fn interaction_available(state: Res<LobbyEntranceState>) -> bool {
    !state.active()
}

fn freeze_text_focus(state: Res<LobbyEntranceState>, mut focus: ResMut<InputFocus>) {
    if state.active() {
        focus.clear();
    }
}

fn advance_entrance(time: Res<Time<Real>>, mut state: ResMut<LobbyEntranceState>) {
    state.advance(time.delta_secs());
}
