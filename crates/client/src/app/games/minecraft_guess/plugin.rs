use super::background::GuessBackgroundPlugin;
use super::graph::{
    GraphArt, GraphCurveMaterial, GraphDragTarget, GraphGeneration, GraphNodeMaterial,
    animate_graph_nodes, animate_graph_panel, begin_generation_notice, graph_pointer_input,
    layout_graph, show_generation_notice, sync_graph_state,
};
use super::process_presence::MinecraftProcessPresencePlugin;
use super::{
    actions::{dispatch_guess_actions, sync_guess_input},
    assets::load_guess_assets,
    controls::{navigate_guess_candidates, sync_guess_button_skins, sync_guess_controls},
    feedback::{
        GuessFeedback, animate_cells, animate_reveal, load_feedback_audio, play_feedback_audio,
        sync_feedback,
    },
    scores::{GuessScorePlayback, animate_scores, sync_score_playback},
    state::{GuessScroll, MinecraftGuessUiState},
    table::{fill_empty_rows, sync_attribute_hints, sync_name_heights, sync_name_widths},
    turn_timer::sync_guess_clock,
};
use crate::app::games::sync_summary_playback;
use crate::app::{
    runtime::ClientUpdateSet,
    shell::{UiActionSet, scroll_modal_content},
};
use bevy::picking::Pickable;
use bevy::prelude::*;
use bevy::ui::UiSystems;
use bevy::ui_render::UiMaterialPlugin;

pub(crate) struct MinecraftGuessPlugin;

impl Plugin for MinecraftGuessPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((GuessBackgroundPlugin, MinecraftProcessPresencePlugin))
            .init_resource::<GraphArt>()
            .init_resource::<GraphGeneration>()
            .add_systems(
                Update,
                begin_generation_notice.in_set(ClientUpdateSet::Input),
            )
            .add_systems(
                Update,
                show_generation_notice.in_set(ClientUpdateSet::Animate),
            )
            .add_plugins((
                UiMaterialPlugin::<GraphCurveMaterial>::default(),
                UiMaterialPlugin::<GraphNodeMaterial>::default(),
            ))
            .register_required_components::<GraphDragTarget, Pickable>()
            .init_resource::<GuessFeedback>()
            .init_resource::<GuessScorePlayback>()
            .init_resource::<MinecraftGuessUiState>()
            .add_observer(navigate_guess_candidates)
            .add_systems(Startup, (load_guess_assets, load_feedback_audio))
            .add_systems(PostUpdate, fill_empty_rows.after(UiSystems::Layout))
            .add_systems(PostUpdate, layout_graph.before(UiSystems::Layout))
            .add_systems(Update, graph_pointer_input.in_set(ClientUpdateSet::Input))
            .add_systems(Update, animate_graph_nodes.in_set(ClientUpdateSet::Animate))
            .add_systems(
                Update,
                animate_graph_panel
                    .in_set(ClientUpdateSet::Animate)
                    .after(animate_cells),
            )
            .add_systems(
                PostUpdate,
                sync_name_widths
                    .after(UiSystems::Content)
                    .before(UiSystems::Layout),
            )
            .add_systems(PostUpdate, sync_name_heights.after(UiSystems::PostLayout))
            .add_systems(
                Update,
                (
                    (sync_feedback, sync_score_playback)
                        .chain()
                        .in_set(ClientUpdateSet::Sync)
                        .before(sync_summary_playback),
                    (
                        play_feedback_audio,
                        animate_cells,
                        animate_scores,
                        animate_reveal,
                    )
                        .chain()
                        .in_set(ClientUpdateSet::Animate),
                    dispatch_guess_actions
                        .in_set(UiActionSet)
                        .in_set(ClientUpdateSet::Input),
                    (
                        sync_guess_input,
                        sync_graph_state,
                        sync_guess_controls,
                        sync_guess_button_skins,
                        sync_guess_clock,
                        sync_attribute_hints,
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
