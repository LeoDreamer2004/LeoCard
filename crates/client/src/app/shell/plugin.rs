use super::{
    ChatPanelState, DeveloperHandInput, PlayErrorToast, PlayerInteractionCooldown, UiActionPlugin,
    UiActionSet, UiState, UiZoom, animate_auto_play_robot_indicators, animate_button_presses,
    animate_chat_bubbles, animate_chat_panel, animate_lobby_seat_hover,
    animate_player_interactions, animate_turn_clocks, chat,
    close_interaction_menu_on_outside_click, dispatch_connection_actions, dispatch_lobby_actions,
    dispatch_navigation_actions, handle_lobby_bot_seat_right_click, handle_text_input,
    play_button_click_sounds, poll_update_events, rebuild_ui, scroll_chat_menus, social,
    sync_avatar_images, sync_chat_messages, sync_chat_panel_text, sync_developer_hand_input_text,
    sync_ime_enabled, sync_interaction_cooldown_masks, sync_opponent_badge_popups,
    sync_player_interactions, sync_update_dialog, tick_player_interaction_cooldown,
    update_button_tints, update_rule_help_tooltips,
};
use crate::app::games::qigui523::{animate_no_legal_response_hint, sync_selection_label};
use crate::app::games::texas_holdem::handle_texas_raise_button_hold;
use crate::app::runtime::ClientUpdateSet;
use bevy::prelude::*;

pub(crate) struct ShellPlugin;

impl Plugin for ShellPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(PlayErrorToast::default())
            .insert_resource(PlayerInteractionCooldown::default())
            .insert_resource(UiState {
                dirty: true,
                ..default()
            })
            .insert_resource(UiZoom::default())
            .insert_resource(ChatPanelState::default())
            .insert_resource(DeveloperHandInput::default())
            .add_plugins(UiActionPlugin)
            .configure_sets(
                Update,
                UiActionSet
                    .after(handle_texas_raise_button_hold)
                    .before(close_interaction_menu_on_outside_click),
            )
            .add_systems(
                Update,
                (
                    social::actions::dispatch_social_actions,
                    chat::actions::dispatch_chat_actions,
                    dispatch_connection_actions,
                    dispatch_navigation_actions,
                    dispatch_lobby_actions,
                )
                    .in_set(UiActionSet),
            )
            .add_systems(
                Update,
                (
                    sync_ime_enabled,
                    handle_text_input,
                    update_button_tints,
                    play_button_click_sounds,
                    animate_button_presses,
                    animate_lobby_seat_hover,
                    handle_lobby_bot_seat_right_click,
                    update_rule_help_tooltips,
                )
                    .chain()
                    .in_set(ClientUpdateSet::Input),
            )
            .add_systems(
                Update,
                (
                    tick_player_interaction_cooldown,
                    close_interaction_menu_on_outside_click,
                    sync_opponent_badge_popups,
                    sync_interaction_cooldown_masks,
                    sync_selection_label,
                    sync_developer_hand_input_text,
                    animate_no_legal_response_hint,
                    sync_chat_messages,
                    sync_player_interactions,
                    sync_avatar_images,
                    poll_update_events,
                    sync_update_dialog,
                    sync_chat_panel_text,
                    scroll_chat_menus,
                )
                    .in_set(ClientUpdateSet::Sync),
            )
            .add_systems(Update, rebuild_ui.in_set(ClientUpdateSet::Rebuild))
            .add_systems(
                Update,
                (
                    animate_turn_clocks,
                    animate_player_interactions,
                    animate_chat_bubbles,
                    animate_chat_panel,
                    animate_auto_play_robot_indicators,
                )
                    .in_set(ClientUpdateSet::Animate),
            );
    }
}
