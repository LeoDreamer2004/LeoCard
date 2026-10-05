use super::advance_mahjong_modal;
use super::{
    MahjongClaimPresentationState, MahjongFanGuideScroll, MahjongFanVoicePlayback,
    MahjongTileMaterial, MahjongUiState, actions, advance_mahjong_claim_presentation,
    animate_mahjong_claim_presentation, animate_mahjong_deal_tiles,
    animate_mahjong_fan_guide_tiles, animate_mahjong_final_summary,
    animate_mahjong_flower_presentations, animate_mahjong_settlement, animate_mahjong_turn_sector,
    animate_mahjong_win_effects, animate_mahjong_win_screen_shake, animate_mahjong_win_tile_shakes,
    animate_mahjong_winning_hands, animate_own_discard, animate_remote_discard,
    apply_automatic_mahjong_action, assets, play_mahjong_fan_voices,
    sync_mahjong_claim_presentation, sync_mahjong_fan_guide, sync_mahjong_hand_tile_materials,
    sync_mahjong_hover_hints,
};
use crate::app::presentation::start_game_intro_finished;
use crate::app::runtime::ClientUpdateSet;
use crate::app::shell::{
    ModalAnimationSet, UiActionSet, animate_cozy_modals, scroll_modal_content,
};
use bevy::prelude::*;

pub(crate) struct MahjongPlugin;

impl Plugin for MahjongPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(UiMaterialPlugin::<MahjongTileMaterial>::default())
            .insert_resource(MahjongClaimPresentationState::default())
            .init_resource::<MahjongFanVoicePlayback>()
            .init_resource::<MahjongUiState>()
            .add_systems(
                Update,
                advance_mahjong_modal.in_set(ModalAnimationSet::Progress),
            )
            .add_systems(Startup, assets::load_mahjong_assets)
            .add_systems(
                Update,
                actions::dispatch_mahjong_actions.in_set(UiActionSet),
            )
            .add_systems(
                Update,
                (
                    sync_mahjong_fan_guide,
                    scroll_modal_content::<MahjongFanGuideScroll>,
                )
                    .chain()
                    .before(animate_cozy_modals)
                    .in_set(ClientUpdateSet::Animate),
            )
            .add_systems(
                Update,
                animate_mahjong_fan_guide_tiles
                    .after(animate_cozy_modals)
                    .in_set(ClientUpdateSet::Animate),
            )
            .add_systems(
                Update,
                (
                    sync_mahjong_hand_tile_materials,
                    sync_mahjong_hover_hints,
                    apply_automatic_mahjong_action
                        .after(actions::dispatch_mahjong_actions)
                        .run_if(start_game_intro_finished),
                )
                    .in_set(ClientUpdateSet::Sync),
            )
            .add_systems(
                Update,
                (
                    animate_mahjong_deal_tiles,
                    animate_own_discard,
                    animate_remote_discard,
                    animate_mahjong_turn_sector,
                    animate_mahjong_winning_hands,
                )
                    .chain()
                    .run_if(start_game_intro_finished)
                    .in_set(ClientUpdateSet::Animate),
            )
            .add_systems(
                Update,
                (
                    sync_mahjong_claim_presentation,
                    advance_mahjong_claim_presentation,
                )
                    .chain()
                    .run_if(start_game_intro_finished)
                    .in_set(ClientUpdateSet::Sync),
            )
            .add_systems(
                Update,
                (
                    play_mahjong_fan_voices,
                    animate_mahjong_settlement,
                    animate_mahjong_final_summary,
                    animate_mahjong_claim_presentation,
                    animate_mahjong_flower_presentations,
                    animate_mahjong_win_effects,
                    animate_mahjong_win_tile_shakes,
                    animate_mahjong_win_screen_shake,
                )
                    .chain()
                    .run_if(start_game_intro_finished)
                    .in_set(ClientUpdateSet::Animate),
            );
    }
}
