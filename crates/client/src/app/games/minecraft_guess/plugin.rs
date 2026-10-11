use super::{
    actions::{dispatch_guess_actions, sync_guess_input},
    assets::load_guess_assets,
    controls::{navigate_guess_candidates, sync_guess_button_skins, sync_guess_controls},
    scores::{GuessScorePlayback, animate_scores, sync_score_playback},
    state::{GuessScroll, MinecraftGuessUiState},
    table::fill_empty_rows,
    turn_timer::sync_guess_clock,
};
use crate::app::{
    runtime::ClientUpdateSet,
    shell::{UiActionSet, scroll_modal_content},
};
use bevy::prelude::*;
use bevy::ui::UiSystems;

pub(crate) struct MinecraftGuessPlugin;

impl Plugin for MinecraftGuessPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<GuessScorePlayback>()
            .init_resource::<MinecraftGuessUiState>()
            .add_observer(navigate_guess_candidates)
            .add_systems(Startup, load_guess_assets)
            .add_systems(PostUpdate, fill_empty_rows.after(UiSystems::Layout))
            .add_systems(
                Update,
                (
                    sync_score_playback
                        .in_set(ClientUpdateSet::Sync)
                        .before(super::super::summary::sync_summary_playback),
                    animate_scores.in_set(ClientUpdateSet::Animate),
                    dispatch_guess_actions
                        .in_set(UiActionSet)
                        .in_set(ClientUpdateSet::Input),
                    (
                        sync_guess_input,
                        sync_guess_controls,
                        sync_guess_button_skins,
                        sync_guess_clock,
                    )
                        .chain()
                        .in_set(ClientUpdateSet::Sync),
                    (scroll_modal_content::<GuessScroll>, retain_guess_scroll)
                        .chain()
                        .in_set(ClientUpdateSet::Input),
                ),
            );
    }
}

fn retain_guess_scroll(
    mut scrolls: Query<(&mut ScrollPosition, &ComputedNode), With<GuessScroll>>,
    mut state: ResMut<MinecraftGuessUiState>,
) {
    if let Ok((mut position, node)) = scrolls.single_mut() {
        if state.scroll_to_latest {
            position.y =
                ((node.content_size().y - node.size().y) * node.inverse_scale_factor()).max(0.0);
            state.scroll_to_latest = false;
        }
        state.scroll_y = position.y;
    }
}
