use super::{
    PlayEffectState, QiGui523UiState, actions, advance_play_effect, animate_bomb_play_effect,
    animate_hand_card_slots, animate_hand_cards, animate_heaven_bomb_play_effect,
    animate_no_legal_response_hint, animate_sequence_play_effect, animate_turn_clocks, assets,
    handle_card_drag_selection, queue_deal_animations, sync_card_drag_preview, sync_play_effect,
    sync_selection_label, sync_turn_timer_label,
};
use crate::app::presentation::play_pending_deal_sounds;
use crate::app::runtime::ClientUpdateSet;
use crate::app::shell::{
    ScoreCaptureEffectState, UiActionSet, animate_play_error_popup, animate_score_capture_effects,
    sync_play_error_toast, sync_score_capture_effect,
};
use bevy::prelude::*;

pub(crate) struct QiGui523Plugin;

impl Plugin for QiGui523Plugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(PlayEffectState::default())
            .insert_resource(ScoreCaptureEffectState::default())
            .init_resource::<QiGui523UiState>()
            .add_systems(Startup, assets::load_qigui523_assets)
            .add_systems(
                Update,
                actions::dispatch_qigui523_actions.in_set(UiActionSet),
            )
            .add_systems(
                Update,
                (handle_card_drag_selection, animate_hand_card_slots)
                    .chain()
                    .in_set(ClientUpdateSet::Input),
            )
            .add_systems(
                Update,
                (
                    sync_turn_timer_label,
                    sync_selection_label,
                    animate_no_legal_response_hint,
                    sync_play_effect,
                    queue_deal_animations,
                    sync_play_error_toast,
                    sync_score_capture_effect,
                )
                    .in_set(ClientUpdateSet::Sync),
            )
            .add_systems(
                Update,
                (
                    animate_hand_cards,
                    advance_play_effect,
                    animate_sequence_play_effect,
                    animate_bomb_play_effect,
                    animate_heaven_bomb_play_effect,
                    animate_turn_clocks,
                    sync_card_drag_preview,
                    play_pending_deal_sounds,
                    animate_play_error_popup,
                    animate_score_capture_effects,
                )
                    .in_set(ClientUpdateSet::Animate),
            );
    }
}
