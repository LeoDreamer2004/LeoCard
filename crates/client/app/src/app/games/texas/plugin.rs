use super::{
    TexasChipTableState, TexasHandGuidePlugin, TexasRaiseHoldState, TexasUiState, TexasVoiceState,
    actions, animate_squid_markers, animate_texas_action_feedback, animate_texas_board_card_backs,
    animate_texas_chip_sprites, animate_texas_flying_card_backs, animate_texas_pot_dividers,
    animate_texas_showdown_reveal, animate_texas_win_rates, handle_texas_raise_button_hold,
    highlight_texas_pot_eligible_players, load_texas_sounds, play_texas_audio_cues,
    sync_texas_chip_state, sync_texas_own_fold_tooltip, sync_texas_spectator, sync_texas_voices,
};
use crate::app::presentation::start_game_intro_finished;
use crate::app::runtime::{ClientUpdateSet, GameEventSource, GameEvents, GameProjectionPlugin};

use crate::app::shell::UiActionSet;
use bevy::prelude::*;
use leocard_protocol::{GameEvent, GameKind, TexasEvent};

pub(crate) struct TexasPlugin;

impl Plugin for TexasPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(GameProjectionPlugin::<GameEvents<TexasEvent>>::default())
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
                handle_texas_raise_button_hold.in_set(ClientUpdateSet::Input),
            )
            .add_systems(
                Update,
                sync_texas_chip_state
                    .run_if(start_game_intro_finished)
                    .in_set(ClientUpdateSet::Sync),
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
                (
                    highlight_texas_pot_eligible_players,
                    sync_texas_own_fold_tooltip,
                    animate_texas_action_feedback,
                )
                    .chain()
                    .run_if(start_game_intro_finished)
                    .in_set(ClientUpdateSet::Animate),
            );
    }
}

impl GameEventSource for TexasEvent {
    const KIND: GameKind = GameKind::Texas;
    fn select(event: &GameEvent) -> Option<&Self> {
        match event {
            GameEvent::Texas(event) => Some(event),
            _ => None,
        }
    }
}
