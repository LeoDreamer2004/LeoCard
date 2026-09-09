use super::{
    TexasChipTableState, TexasHoldemUiState, TexasRaiseHoldState, actions,
    animate_texas_action_feedback, animate_texas_board_card_backs, animate_texas_board_card_flips,
    animate_texas_chip_sprites, animate_texas_deal_cards, animate_texas_flying_card_backs,
    animate_texas_pot_dividers, animate_texas_showdown_reveal, assets,
    handle_texas_raise_button_hold, highlight_texas_pot_eligible_players, play_texas_audio_cues,
    sync_texas_chip_state, sync_texas_own_fold_tooltip,
};
use crate::app::runtime::ClientUpdateSet;
use crate::app::shell::UiActionSet;
use bevy::prelude::*;

pub(crate) struct TexasHoldemPlugin;

impl Plugin for TexasHoldemPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(TexasRaiseHoldState::default())
            .insert_resource(TexasChipTableState::default())
            .init_resource::<TexasHoldemUiState>()
            .add_systems(Startup, assets::load_texas_holdem_assets)
            .add_systems(
                Update,
                actions::dispatch_texas_holdem_actions.in_set(UiActionSet),
            )
            .add_systems(
                Update,
                handle_texas_raise_button_hold.in_set(ClientUpdateSet::Input),
            )
            .add_systems(Update, sync_texas_chip_state.in_set(ClientUpdateSet::Sync))
            .add_systems(
                Update,
                play_texas_audio_cues.in_set(ClientUpdateSet::Animate),
            )
            .add_systems(
                Update,
                (
                    animate_texas_deal_cards,
                    animate_texas_flying_card_backs,
                    animate_texas_board_card_backs,
                    animate_texas_board_card_flips,
                    animate_texas_showdown_reveal,
                    animate_texas_chip_sprites,
                    animate_texas_pot_dividers,
                )
                    .chain()
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
                    .in_set(ClientUpdateSet::Animate),
            );
    }
}
