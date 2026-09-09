use super::{
    ShengjiPresentationState, ShengjiScoreCaptureEffectState, ShengjiSelectionSync,
    ShengjiSettlementAnimation, ShengjiUiState, actions, advance_shengji_presentation,
    animate_shengji_bottom_flip_markers, animate_shengji_failed_throw_cards,
    animate_shengji_failed_throw_labels, animate_shengji_hand_card_slots,
    animate_shengji_hand_cards, animate_shengji_power_outage_markers, animate_shengji_presentation,
    animate_shengji_score_absorbs, animate_shengji_score_capture_score,
    animate_shengji_settlement_visuals, animate_shengji_throw_penalty_floats,
    animate_shengji_throw_penalty_score_pulses, assets, handle_shengji_card_drag_selection,
    play_shengji_audio_cues, queue_shengji_deal_animations, spawn_shengji_settlement_absorption,
    sync_shengji_bidding_countdown, sync_shengji_card_drag_preview, sync_shengji_phase_selection,
    sync_shengji_presentation, sync_shengji_score_capture_effect,
    update_shengji_settlement_animation,
};
use crate::app::runtime::ClientUpdateSet;
use crate::app::shell::UiActionSet;
use bevy::prelude::*;

pub(crate) struct ShengjiPlugin;

impl Plugin for ShengjiPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(ShengjiScoreCaptureEffectState::default())
            .insert_resource(ShengjiSettlementAnimation::default())
            .insert_resource(ShengjiPresentationState::default())
            .insert_resource(ShengjiSelectionSync::default())
            .init_resource::<ShengjiUiState>()
            .add_systems(Startup, assets::load_shengji_assets)
            .add_systems(
                Update,
                actions::dispatch_shengji_actions.in_set(UiActionSet),
            )
            .add_systems(
                Update,
                (
                    handle_shengji_card_drag_selection,
                    animate_shengji_hand_card_slots,
                    sync_shengji_card_drag_preview,
                )
                    .chain()
                    .in_set(ClientUpdateSet::Input),
            )
            .add_systems(
                Update,
                (
                    sync_shengji_phase_selection,
                    sync_shengji_presentation,
                    update_shengji_settlement_animation,
                )
                    .chain()
                    .in_set(ClientUpdateSet::Sync),
            )
            .add_systems(
                Update,
                sync_shengji_score_capture_effect.in_set(ClientUpdateSet::Sync),
            )
            .add_systems(
                Update,
                (queue_shengji_deal_animations, animate_shengji_hand_cards)
                    .chain()
                    .in_set(ClientUpdateSet::Animate),
            )
            .add_systems(
                Update,
                (
                    sync_shengji_bidding_countdown,
                    (
                        animate_shengji_failed_throw_cards,
                        animate_shengji_failed_throw_labels,
                        animate_shengji_throw_penalty_floats,
                        animate_shengji_throw_penalty_score_pulses,
                    )
                        .chain(),
                    animate_shengji_settlement_visuals,
                    animate_shengji_score_capture_score,
                    advance_shengji_presentation,
                    animate_shengji_presentation,
                    animate_shengji_bottom_flip_markers,
                    animate_shengji_power_outage_markers,
                    play_shengji_audio_cues,
                    spawn_shengji_settlement_absorption,
                    animate_shengji_score_absorbs,
                )
                    .in_set(ClientUpdateSet::Animate),
            );
    }
}
