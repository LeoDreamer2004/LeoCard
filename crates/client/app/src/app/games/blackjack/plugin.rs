use super::actions::dispatch_blackjack_actions;
use super::chips::register_wager_properties;
use super::feedback::{BlackjackFeedback, load_sounds, play_feedback};
use super::playback::{BlackjackCardTiming, sync_blackjack_playback};
use super::{BlackjackPlayback, BlackjackUiState};
use crate::app::games::sync_summary_playback;
use crate::app::presentation::start_game_intro_finished;
use crate::app::runtime::ClientUpdateSet;
use crate::app::shell::UiActionSet;
use bevy::prelude::*;
use leocard_ui::UiSceneProperties;

pub(crate) struct BlackjackPlugin;

impl Plugin for BlackjackPlugin {
    fn build(&self, app: &mut App) {
        let mut properties = app.world_mut().resource_mut::<UiSceneProperties>();
        properties.preserve::<BlackjackCardTiming>();
        register_wager_properties(&mut properties);
        app.init_resource::<BlackjackUiState>()
            .init_resource::<BlackjackFeedback>()
            .init_resource::<BlackjackPlayback>()
            .add_systems(
                Update,
                sync_blackjack_playback
                    .in_set(ClientUpdateSet::Sync)
                    .before(sync_summary_playback)
                    .run_if(start_game_intro_finished),
            )
            .add_systems(Startup, load_sounds)
            .add_systems(Update, dispatch_blackjack_actions.in_set(UiActionSet))
            .add_systems(
                Update,
                play_feedback
                    .in_set(ClientUpdateSet::Animate)
                    .run_if(start_game_intro_finished),
            );
    }
}
