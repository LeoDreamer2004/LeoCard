use super::{
    UnoAudioState, UnoPaletteMaterial, UnoPresentationState, UnoUiState, actions,
    animate_uno_flip_effects, animate_uno_flying_cards, animate_uno_hand_cards,
    animate_uno_palette_color_rings, animate_uno_palette_effects, animate_uno_palette_particles,
    animate_uno_palette_selected_sectors, animate_uno_reverse_effects,
    animate_uno_swap_target_panels, assets, play_uno_audio_cues, play_uno_card_selection_sounds,
    spawn_uno_presentation_effects, sync_uno_discard_reveal, sync_uno_extension_card_help,
    sync_uno_presentation,
};
use crate::app::runtime::ClientUpdateSet;
use crate::app::shell::UiActionSet;
use bevy::prelude::*;

pub(crate) struct UnoPlugin;

impl Plugin for UnoPlugin {
    fn build(&self, app: &mut App) {
        app.init_asset::<UnoPaletteMaterial>()
            .add_plugins(UiMaterialPlugin::<UnoPaletteMaterial>::default())
            .insert_resource(UnoPresentationState::default())
            .insert_resource(UnoAudioState::default())
            .init_resource::<UnoUiState>()
            .add_systems(Startup, assets::load_uno_assets)
            .add_systems(Update, actions::dispatch_uno_actions.in_set(UiActionSet))
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
                (sync_uno_extension_card_help, sync_uno_presentation).in_set(ClientUpdateSet::Sync),
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
                    .in_set(ClientUpdateSet::Animate),
            );
    }
}
