use super::{
    MahjongClaimPresentationState, MahjongTileMaterial, MahjongUiState, actions,
    advance_mahjong_claim_presentation, animate_mahjong_claim_presentation,
    animate_mahjong_deal_tiles, animate_mahjong_flower_presentations, animate_mahjong_turn_arrows,
    animate_mahjong_win_effects, animate_mahjong_win_screen_shake, animate_mahjong_winning_hands,
    assets, sync_mahjong_claim_presentation, sync_mahjong_hand_tile_materials,
};
use crate::app::runtime::ClientUpdateSet;
use crate::app::shell::UiActionSet;
use bevy::prelude::*;

pub(crate) struct MahjongPlugin;

impl Plugin for MahjongPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(UiMaterialPlugin::<MahjongTileMaterial>::default())
            .insert_resource(MahjongClaimPresentationState::default())
            .init_resource::<MahjongUiState>()
            .add_systems(Startup, assets::load_mahjong_assets)
            .add_systems(
                Update,
                actions::dispatch_mahjong_actions.in_set(UiActionSet),
            )
            .add_systems(
                Update,
                (sync_mahjong_hand_tile_materials,).in_set(ClientUpdateSet::Sync),
            )
            .add_systems(
                Update,
                (
                    animate_mahjong_deal_tiles,
                    animate_mahjong_turn_arrows,
                    animate_mahjong_winning_hands,
                )
                    .chain()
                    .in_set(ClientUpdateSet::Animate),
            )
            .add_systems(
                Update,
                (
                    sync_mahjong_claim_presentation,
                    advance_mahjong_claim_presentation,
                )
                    .chain()
                    .in_set(ClientUpdateSet::Sync),
            )
            .add_systems(
                Update,
                (
                    animate_mahjong_claim_presentation,
                    animate_mahjong_flower_presentations,
                    animate_mahjong_win_effects,
                    animate_mahjong_win_screen_shake,
                )
                    .chain()
                    .in_set(ClientUpdateSet::Animate),
            );
    }
}
