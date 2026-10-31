use super::drag::{UnoDragGuideMaterial, UnoDragState, render_uno_drag, update_uno_drag};
use super::jump_in_device::update_jump_in_device;
use super::{
    UnoAudioState, UnoJumpInDevice, UnoPaletteMaterial, UnoPresentationState, UnoUiState, actions,
    advance_uno_modal, animate_uno_flip_effects, animate_uno_flying_cards, animate_uno_hand_cards,
    animate_uno_palette_color_rings, animate_uno_palette_effects, animate_uno_palette_particles,
    animate_uno_palette_selected_sectors, animate_uno_reverse_effects,
    animate_uno_swap_target_panels, assets, play_uno_audio_cues, play_uno_card_selection_sounds,
    spawn_uno_presentation_effects, sync_uno_discard_reveal, sync_uno_extension_card_help,
    sync_uno_presentation,
};
use crate::app::presentation::start_game_intro_finished;
use crate::app::runtime::{ClientUpdateSet, GameEventSource, GameEvents, GameProjectionPlugin};

use crate::app::shell::{ModalAnimationSet, UiActionSet};
use bevy::prelude::*;
use leocard_protocol::{GameEvent, GameKind, UnoEvent};

pub(crate) struct UnoPlugin;

impl Plugin for UnoPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            super::notices::sync_notices.in_set(ClientUpdateSet::Project),
        )
        .add_plugins(GameProjectionPlugin::<GameEvents<UnoEvent>>::default())
        .init_asset::<UnoPaletteMaterial>()
        .add_plugins(UiMaterialPlugin::<UnoPaletteMaterial>::default())
        .add_plugins(UiMaterialPlugin::<UnoDragGuideMaterial>::default())
        .init_resource::<UnoDragState>()
        .insert_resource(UnoPresentationState::default())
        .insert_resource(UnoAudioState::default())
        .init_resource::<UnoUiState>()
        .init_resource::<UnoJumpInDevice>()
        .add_systems(
            Update,
            advance_uno_modal.in_set(ModalAnimationSet::Progress),
        )
        .add_systems(Startup, assets::load_uno_assets)
        .add_systems(Update, actions::dispatch_uno_actions.in_set(UiActionSet))
        .add_systems(
            Update,
            update_uno_drag
                .in_set(ClientUpdateSet::Input)
                .before(UiActionSet),
        )
        .add_systems(Update, render_uno_drag.in_set(ClientUpdateSet::Animate))
        .add_systems(Update, update_jump_in_device.in_set(ClientUpdateSet::Sync))
        .add_systems(
            Update,
            (
                play_uno_card_selection_sounds,
                animate_uno_hand_cards,
                animate_uno_swap_target_panels,
            )
                .in_set(ClientUpdateSet::Input),
        )
        .add_systems(
            Update,
            (
                sync_uno_extension_card_help,
                sync_uno_presentation.run_if(start_game_intro_finished),
            )
                .in_set(ClientUpdateSet::Sync),
        )
        .add_systems(
            Update,
            (
                spawn_uno_presentation_effects,
                play_uno_audio_cues,
                (
                    (animate_uno_flying_cards, sync_uno_discard_reveal).chain(),
                    animate_uno_palette_effects,
                    animate_uno_palette_selected_sectors,
                    animate_uno_palette_color_rings,
                    animate_uno_palette_particles,
                    animate_uno_reverse_effects,
                    animate_uno_flip_effects,
                ),
            )
                .chain()
                .run_if(start_game_intro_finished)
                .in_set(ClientUpdateSet::Animate),
        );
    }
}

impl GameEventSource for UnoEvent {
    const KIND: GameKind = GameKind::Uno;
    fn select(event: &GameEvent) -> Option<&Self> {
        match event {
            GameEvent::Uno(event) => Some(event),
            _ => None,
        }
    }
}
