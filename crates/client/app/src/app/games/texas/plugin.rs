use super::betting::{
    focus_texas_raise_input, handle_texas_raise_slider, sync_texas_raise_controls,
    sync_texas_raise_input, sync_texas_raise_value,
};
use super::{
    TexasChipTableState, TexasChipUpdates, TexasHandGuidePlugin, TexasRaiseHoldState, TexasUiState,
    TexasVoiceState, actions, animate_squid_markers, animate_texas_action_feedback,
    animate_texas_board_card_backs, animate_texas_chip_sprites, animate_texas_flying_card_backs,
    animate_texas_pot_dividers, animate_texas_runout_cards, animate_texas_showdown_reveal,
    animate_texas_win_rates, handle_texas_raise_button_hold, load_texas_sounds,
    play_texas_audio_cues, sync_texas_chip_state, sync_texas_own_fold_tooltip,
    sync_texas_spectator, sync_texas_voices,
};
use crate::app::games::sync_summary_playback;
use crate::app::presentation::start_game_intro_finished;
use crate::app::runtime::{ClientUpdateSet, GameProjectionPlugin};
use crate::app::shell::UiActionSet;
use bevy::prelude::*;
use bevy::ui::UiSystems;
use leocard_ui::TextInputSet;

pub(crate) struct TexasPlugin;

impl Plugin for TexasPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(GameProjectionPlugin::<TexasChipUpdates>::default())
            .insert_resource(TexasRaiseHoldState::default())
            .insert_resource(TexasChipTableState::default())
            .init_resource::<TexasUiState>()
            .add_plugins(TexasHandGuidePlugin)
            .init_resource::<TexasVoiceState>()
            .add_systems(
                Update,
                (sync_texas_voices, sync_texas_spectator).in_set(ClientUpdateSet::Sync),
            )
            .add_systems(
                Update,
                animate_texas_win_rates.in_set(ClientUpdateSet::Animate),
            )
            .add_systems(Startup, load_texas_sounds)
            .configure_sets(Update, UiActionSet.after(handle_texas_raise_button_hold))
            .add_systems(Update, actions::dispatch_texas_actions.in_set(UiActionSet))
            .add_systems(
                Update,
                (handle_texas_raise_button_hold, handle_texas_raise_slider)
                    .in_set(ClientUpdateSet::Input),
            )
            .add_systems(
                Update,
                focus_texas_raise_input.after(ClientUpdateSet::Compose),
            )
            .add_systems(
                Update,
                (sync_texas_raise_controls, sync_texas_raise_value)
                    .after(ClientUpdateSet::Compose)
                    .after(UiActionSet),
            )
            .add_systems(
                PostUpdate,
                sync_texas_raise_input
                    .after(TextInputSet::Publish)
                    .before(UiSystems::PostLayout),
            )
            .add_systems(
                Update,
                sync_texas_chip_state
                    .run_if(start_game_intro_finished)
                    .in_set(ClientUpdateSet::Sync)
                    .before(sync_summary_playback),
            )
            .add_systems(
                Update,
                play_texas_audio_cues
                    .run_if(start_game_intro_finished)
                    .in_set(ClientUpdateSet::Animate),
            )
            .add_systems(
                Update,
                (
                    animate_texas_flying_card_backs,
                    animate_texas_board_card_backs,
                    animate_texas_runout_cards,
                    animate_texas_showdown_reveal,
                    animate_texas_chip_sprites,
                    animate_squid_markers,
                    animate_texas_pot_dividers,
                )
                    .chain()
                    .run_if(start_game_intro_finished)
                    .in_set(ClientUpdateSet::Animate),
            )
            .add_systems(
                Update,
                (sync_texas_own_fold_tooltip, animate_texas_action_feedback)
                    .chain()
                    .run_if(start_game_intro_finished)
                    .in_set(ClientUpdateSet::Animate),
            );
    }
}
