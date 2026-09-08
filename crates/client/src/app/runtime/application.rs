//! Bevy 插件安装、资源初始化与系统调度。

use super::*;
use bevy::asset::AssetPlugin;
use bevy::audio::{GlobalVolume, Volume};
use bevy::log::{DEFAULT_FILTER, LogPlugin};
use bevy::window::WindowResizeConstraints;
use leocard_client::PlayerIdentity;
use leocard_protocol::PlayerGameProfiles;
use std::collections::HashSet;

pub fn launch() {
    let mut form = ConnectionForm::default();
    let profile = match LocalPlayerProfile::load_or_create() {
        Ok(profile) => profile,
        Err(error) => {
            form.error = Some(error);
            LocalPlayerProfile {
                identity: PlayerIdentity::generate()
                    .expect("the operating system provides entropy"),
                rating: PlayerRatingProfile {
                    reference_points: 0,
                    completed_games: 0,
                    applied_matches: HashSet::new(),
                    last_change: None,
                },
                game_profiles: PlayerGameProfiles::default(),
            }
        }
    };
    let audio_volume = form.audio_volume;
    let mut app = App::new();
    configure_runtime_asset_source(&mut app);
    app.insert_resource(ClearColor(TABLE_BG))
        .insert_resource(form)
        .insert_resource(GlobalVolume::new(Volume::Linear(audio_volume)))
        .insert_resource(profile)
        .insert_resource(AvatarImages::default())
        .insert_resource(AvatarPicker::default())
        .insert_resource(TableFeltPicker::default())
        .insert_resource(TableAppearance::default())
        .insert_resource(PlayErrorToast::default())
        .insert_resource(GameSummaryAnimation::default())
        .insert_resource(TexasRaiseHoldState::default())
        .insert_resource(PlayEffectState::default())
        .insert_resource(ScoreCaptureEffectState::default())
        .insert_resource(ShengjiScoreCaptureEffectState::default())
        .insert_resource(ShengjiSettlementAnimation::default())
        .insert_resource(ShengjiPresentationState::default())
        .insert_resource(MahjongClaimPresentationState::default())
        .insert_resource(UnoPresentationState::default())
        .insert_resource(UnoAudioState::default())
        .insert_resource(StartGameSeatTransition::default())
        .insert_resource(TurnBorderAnimationState::default())
        .insert_resource(PlayerInteractionCooldown::default())
        .insert_resource(UiState {
            dirty: true,
            ..default()
        })
        .insert_resource(UiZoom::default())
        .insert_resource(CardDragSelection::default())
        .insert_resource(ChatPanelState::default())
        .insert_resource(DeveloperHandInput::default())
        .insert_resource(TexasChipTableState::default())
        .insert_resource(UpdateManager::default())
        .add_plugins(
            DefaultPlugins
                .set(LogPlugin {
                    // Parley deliberately uses ICU4X's non-complex-script segmenter.
                    // Chinese text then falls back correctly, but ICU logs a warning for
                    // every layout pass because no CJK dictionary was requested.
                    filter: format!("{DEFAULT_FILTER}icu_provider::error=error"),
                    ..default()
                })
                .set(AssetPlugin {
                    file_path: asset_file_path(),
                    ..default()
                })
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "LeoCard".to_owned(),
                        resolution: (DESIGN_WIDTH as u32, DESIGN_HEIGHT as u32).into(),
                        resize_constraints: WindowResizeConstraints {
                            min_width: 640.0,
                            min_height: 400.0,
                            ..default()
                        },
                        resizable: true,
                        ..default()
                    }),
                    ..default()
                }),
        )
        .add_plugins(UiMaterialPlugin::<TableBackgroundMaterial>::default())
        .add_plugins(UiMaterialPlugin::<TurnBorderMaterial>::default())
        .add_plugins(UiMaterialPlugin::<UnoPaletteMaterial>::default())
        .add_plugins(UiMaterialPlugin::<MahjongTileMaterial>::default())
        .add_plugins(UiActionPlugin)
        .configure_sets(
            Update,
            UiActionSet
                .after(handle_texas_raise_button_hold)
                .before(close_interaction_menu_on_outside_click),
        )
        .add_systems(Startup, (setup_camera, load_ui_assets))
        .add_systems(Update, set_app_window_icon)
        .add_systems(
            Update,
            (
                animate_mahjong_deal_tiles,
                sync_mahjong_hand_tile_materials,
                animate_mahjong_turn_arrows,
                animate_mahjong_winning_hands,
            )
                .chain(),
        )
        .add_systems(
            Update,
            (
                (
                    update_ui_scale,
                    sync_ime_enabled,
                    handle_text_input,
                    handle_avatar_drop,
                    poll_avatar_picker,
                    poll_table_felt_picker,
                    sync_table_appearance,
                    update_button_tints,
                    play_button_click_sounds,
                    play_uno_card_selection_sounds,
                    animate_button_presses,
                    animate_lobby_seat_hover,
                    handle_lobby_bot_seat_right_click,
                    update_rule_help_tooltips,
                    sync_uno_extension_card_help,
                    handle_table_appearance_sliders,
                    handle_card_drag_selection,
                    animate_hand_card_slots,
                    handle_shengji_card_drag_selection,
                    animate_shengji_hand_card_slots,
                )
                    .chain(),
                (
                    (
                        (animate_hand_cards, animate_uno_hand_cards).chain(),
                        (animate_uno_swap_target_panels, animate_turn_clocks),
                        tick_player_interaction_cooldown,
                        (sync_card_drag_preview, sync_shengji_card_drag_preview).chain(),
                        handle_texas_raise_button_hold,
                        close_interaction_menu_on_outside_click,
                        sync_opponent_badge_popups,
                        sync_interaction_cooldown_masks,
                        sync_selection_label,
                        sync_developer_hand_input_text,
                        animate_no_legal_response_hint,
                        (
                            poll_network,
                            sync_uno_presentation,
                            sync_shengji_presentation,
                            sync_mahjong_claim_presentation,
                            advance_mahjong_claim_presentation,
                            update_shengji_settlement_animation,
                        )
                            .chain(),
                        (sync_texas_chip_state, play_texas_audio_cues).chain(),
                        sync_chat_messages,
                        sync_player_interactions,
                        (sync_score_capture_effect, sync_shengji_score_capture_effect).chain(),
                        animate_player_interactions,
                        (
                            animate_score_capture_effects,
                            animate_shengji_score_capture_score,
                        )
                            .chain(),
                        animate_chat_bubbles,
                    )
                        .chain(),
                    (
                        sync_turn_timer_label,
                        sync_play_error_toast,
                        animate_play_error_popup,
                        sync_play_effect,
                        advance_play_effect,
                        animate_sequence_play_effect,
                        animate_bomb_play_effect,
                        animate_heaven_bomb_play_effect,
                        (
                            update_summary_animation,
                            animate_game_summary_visuals,
                            animate_summary_scores,
                            animate_signed_summary_scores,
                        )
                            .chain(),
                        (
                            queue_deal_animations,
                            queue_shengji_deal_animations,
                            animate_shengji_hand_cards,
                        )
                            .chain(),
                        (
                            animate_texas_deal_cards,
                            animate_texas_flying_card_backs,
                            animate_texas_board_card_backs,
                            animate_texas_board_card_flips,
                            animate_texas_showdown_reveal,
                            animate_texas_chip_sprites,
                            animate_texas_pot_dividers,
                        )
                            .chain(),
                        (
                            highlight_texas_pot_eligible_players,
                            sync_texas_own_fold_tooltip,
                            animate_texas_action_feedback,
                        )
                            .chain(),
                        play_pending_deal_sounds,
                        animate_chat_panel,
                        animate_auto_play_robot_indicators,
                        sync_chat_panel_text,
                        scroll_chat_menus,
                        (
                            sync_avatar_images,
                            poll_update_events,
                            rebuild_ui,
                            (
                                animate_mahjong_claim_presentation,
                                animate_mahjong_flower_presentations,
                                animate_mahjong_win_effects,
                                animate_mahjong_win_screen_shake,
                            )
                                .chain(),
                            spawn_uno_presentation_effects,
                            play_uno_audio_cues,
                            (
                                (animate_uno_flying_cards, sync_uno_discard_reveal).chain(),
                                animate_uno_palette_effects,
                                animate_uno_palette_selected_sectors,
                                animate_uno_palette_color_rings,
                                animate_uno_palette_particles,
                                animate_uno_reverse_effects,
                                animate_uno_flip_effects,
                            ),
                            sync_update_dialog,
                            sync_shengji_bidding_countdown,
                            (
                                animate_shengji_failed_throw_cards,
                                animate_shengji_failed_throw_labels,
                                animate_shengji_throw_penalty_floats,
                                animate_shengji_throw_penalty_score_pulses,
                            )
                                .chain(),
                            animate_shengji_settlement_visuals,
                            advance_shengji_presentation,
                            animate_shengji_presentation,
                            animate_shengji_bottom_flip_markers,
                            animate_shengji_power_outage_markers,
                            play_shengji_audio_cues,
                            spawn_shengji_settlement_absorption,
                            animate_shengji_score_absorbs,
                            animate_start_game_seat_transition,
                            animate_turn_border_traces,
                        )
                            .chain(),
                    )
                        .chain(),
                )
                    .chain(),
            )
                .chain(),
        )
        .run();
}
