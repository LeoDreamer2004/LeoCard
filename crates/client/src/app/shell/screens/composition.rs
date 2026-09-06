//! 根据网络模型与页面状态装配当前的顶层界面。

use super::*;
use bevy::ecs::system::SystemParam;
use leocard_protocol::{GameKind, UnoPendingSwapView};

#[derive(SystemParam)]
pub struct VisualAssets<'w> {
    pub ui: Res<'w, UiAssets>,
    pub avatars: Res<'w, AvatarImages>,
    pub table: Res<'w, TableAppearance>,
    pub play_error: Res<'w, PlayErrorToast>,
    pub game_summary: Res<'w, GameSummaryAnimation>,
    pub play_effect: Res<'w, PlayEffectState>,
    pub score_capture: Res<'w, ScoreCaptureEffectState>,
    pub shengji_score_capture: Res<'w, ShengjiScoreCaptureEffectState>,
    pub shengji_settlement: Res<'w, ShengjiSettlementAnimation>,
    pub shengji_presentation: Res<'w, ShengjiPresentationState>,
    pub mahjong_claim_presentation: Res<'w, MahjongClaimPresentationState>,
    pub start_game_transition: Res<'w, StartGameSeatTransition>,
    pub texas_chips: Res<'w, TexasChipTableState>,
    pub updater: Res<'w, UpdateManager>,
}

#[derive(SystemParam)]
pub struct ScreenResources<'w> {
    client: Option<Res<'w, ClientResource>>,
    form: Res<'w, ConnectionForm>,
    profile: Res<'w, LocalPlayerProfile>,
    chat: Res<'w, ChatPanelState>,
    developer_hand: Res<'w, DeveloperHandInput>,
    ui: ResMut<'w, UiState>,
}

#[derive(SystemParam)]
pub struct ScreenMaterials<'w> {
    table: ResMut<'w, Assets<TableBackgroundMaterial>>,
    mahjong_tiles: ResMut<'w, Assets<MahjongTileMaterial>>,
    turn_borders: ResMut<'w, Assets<TurnBorderMaterial>>,
}

pub fn render_ui(
    mut commands: Commands,
    resources: ScreenResources,
    visuals: VisualAssets,
    materials: ScreenMaterials,
    old_roots: Query<Entity, With<UiRoot>>,
) {
    let ScreenResources {
        client,
        form,
        profile,
        chat,
        developer_hand,
        mut ui,
    } = resources;
    let ScreenMaterials {
        table: mut table_materials,
        mahjong_tiles: mut mahjong_tile_materials,
        turn_borders: mut turn_border_materials,
    } = materials;
    if !ui.dirty {
        return;
    }
    ui.dirty = false;
    for entity in &old_roots {
        commands.entity(entity).despawn();
    }
    let in_uno_lobby = client
        .as_deref()
        .and_then(|client| client.0.model().lobby())
        .is_some_and(|lobby| lobby.game == GameKind::Uno);
    if !in_uno_lobby {
        ui.uno.mode_menu_open = false;
        ui.uno.expansion_settings_open = false;
    }

    if let Some(game) = client
        .as_deref()
        .and_then(|client| client.0.model().qigui523_game())
    {
        ui.qigui523
            .selected
            .retain(|card| game.your_hand.contains(card));
        ui.qigui523
            .card_animations
            .retain(|card, _| game.your_hand.contains(card));
        if ui
            .social
            .interaction_menu_open
            .is_some_and(|open| !game.players.iter().any(|player| player.id == open))
        {
            ui.social.interaction_menu_open = None;
        }
    } else if let Some(game) = client
        .as_deref()
        .and_then(|client| client.0.model().texas_holdem_game())
    {
        ui.qigui523.selected.clear();
        ui.qigui523.card_animations.clear();
        ui.qigui523.observed_hand.clear();
        ui.qigui523.greedy_hint.reset();
        if ui
            .social
            .interaction_menu_open
            .is_some_and(|open| !game.players.iter().any(|player| player.id == open))
        {
            ui.social.interaction_menu_open = None;
        }
    } else if let Some(game) = client
        .as_deref()
        .and_then(|client| client.0.model().shengji_game())
    {
        ui.qigui523.selected.clear();
        ui.qigui523.card_animations.clear();
        ui.qigui523.observed_hand.clear();
        ui.qigui523.greedy_hint.reset();
        ui.shengji
            .selected
            .retain(|card| game.your_hand.contains(card));
        if ui
            .social
            .interaction_menu_open
            .is_some_and(|open| !game.players.iter().any(|player| player.id == open))
        {
            ui.social.interaction_menu_open = None;
        }
    } else if let Some(game) = client
        .as_deref()
        .and_then(|client| client.0.model().uno_game())
    {
        ui.qigui523.selected.clear();
        ui.qigui523.card_animations.clear();
        ui.qigui523.observed_hand.clear();
        ui.shengji.selected.clear();
        ui.shengji.card_animations.clear();
        ui.shengji.observed_hand.clear();
        ui.qigui523.greedy_hint.reset();
        ui.uno.selected.retain(|card| game.your_hand.contains(card));
        if let Some(card) = game.your_jump_in_card {
            ui.uno.selected.clear();
            ui.uno.selected.insert(card);
        } else if game.current_player != Some(game.you) {
            ui.uno.selected.clear();
        }
        ui.uno
            .card_animations
            .retain(|card, _| game.your_hand.contains(card));
        let selecting_swap_targets = matches!(
            game.pending_swap,
            Some(UnoPendingSwapView::SwapOneTarget { player })
                | Some(UnoPendingSwapView::ForceTrade { player })
                | Some(UnoPendingSwapView::SevenSwap { player }) if player == game.you
        );
        if !selecting_swap_targets {
            ui.uno.swap_targets.clear();
        } else {
            ui.social.interaction_menu_open = None;
            ui.uno.swap_targets.retain(|target| {
                game.players
                    .iter()
                    .any(|player| player.id == *target && !player.eliminated)
            });
        }
        if ui
            .uno
            .color_choice
            .is_some_and(|card| !game.your_hand.contains(&card))
        {
            ui.uno.color_choice = None;
        }
        if ui
            .social
            .interaction_menu_open
            .is_some_and(|open| !game.players.iter().any(|player| player.id == open))
        {
            ui.social.interaction_menu_open = None;
        }
    } else if let Some(game) = client
        .as_deref()
        .and_then(|client| client.0.model().mahjong_game())
    {
        ui.qigui523.selected.clear();
        ui.qigui523.card_animations.clear();
        ui.qigui523.observed_hand.clear();
        ui.shengji.selected.clear();
        ui.shengji.card_animations.clear();
        ui.shengji.observed_hand.clear();
        ui.uno.selected.clear();
        ui.uno.card_animations.clear();
        ui.qigui523.greedy_hint.reset();
        if ui
            .social
            .interaction_menu_open
            .is_some_and(|open| !game.players.iter().any(|player| player.id == open))
        {
            ui.social.interaction_menu_open = None;
        }
    } else {
        ui.qigui523.selected.clear();
        ui.qigui523.card_animations.clear();
        ui.qigui523.observed_hand.clear();
        ui.qigui523.greedy_hint.reset();
        ui.social.interaction_menu_open = None;
        ui.texas_holdem.observed_match = None;
        ui.texas_holdem.observed_hand_number = 0;
        ui.texas_holdem.observed_community_len = 0;
        ui.shengji.selected.clear();
        ui.shengji.card_animations.clear();
        ui.shengji.observed_hand.clear();
        ui.shengji.observed_match = None;
        ui.shengji.observed_hand_number = 0;
        ui.shengji.buried_open = false;
        ui.uno.swap_targets.clear();
        ui.uno.color_choice = None;
        ui.uno.selected.clear();
        ui.uno.card_animations.clear();
    }

    let root = commands
        .spawn((
            UiRoot,
            Node {
                width: percent(100),
                height: percent(100),
                flex_direction: FlexDirection::Column,
                ..default()
            },
        ))
        .id();

    add_header(
        &mut commands,
        root,
        client.as_deref(),
        &form,
        &visuals.ui,
        &visuals.avatars,
    );
    if let Some(client) = client.as_deref() {
        if let Some(lobby) = client.0.model().lobby() {
            render_lobby(
                &mut commands,
                root,
                client,
                lobby,
                &ui,
                &visuals.ui,
                &visuals.avatars,
            );
        } else if let Some(game) = client.0.model().qigui523_game() {
            let mut table_visuals = TableVisualContext {
                assets: &visuals.ui,
                avatars: &visuals.avatars,
                appearance: &visuals.table,
                brightness: form.table_brightness,
                vignette: form.table_vignette,
                table_materials: &mut table_materials,
                game_summary: &visuals.game_summary,
                play_effect: &visuals.play_effect,
                score_capture: &visuals.score_capture,
                start_game_transition: &visuals.start_game_transition,
                turn_border_materials: &mut turn_border_materials,
            };
            render_table(
                &mut commands,
                root,
                client,
                game,
                &ui,
                &chat,
                &developer_hand,
                &mut table_visuals,
            );
        } else if let Some(game) = client.0.model().texas_holdem_game() {
            render_texas_holdem_table(
                &mut commands,
                root,
                client,
                game,
                &mut ui,
                &chat,
                TexasTableVisuals {
                    assets: &visuals.ui,
                    avatars: &visuals.avatars,
                    appearance: &visuals.table,
                    brightness: form.table_brightness,
                    vignette: form.table_vignette,
                    table_materials: &mut table_materials,
                    turn_border_materials: &mut turn_border_materials,
                    start_game_transition: &visuals.start_game_transition,
                    chip_state: &visuals.texas_chips,
                    game_summary: &visuals.game_summary,
                },
            );
        } else if let Some(game) = client.0.model().shengji_game() {
            render_shengji_table(
                &mut commands,
                root,
                client,
                game,
                &mut ui,
                &chat,
                ShengjiTableVisuals {
                    assets: &visuals.ui,
                    avatars: &visuals.avatars,
                    appearance: &visuals.table,
                    brightness: form.table_brightness,
                    vignette: form.table_vignette,
                    table_materials: &mut table_materials,
                    turn_border_materials: &mut turn_border_materials,
                    start_game_transition: &visuals.start_game_transition,
                    score_capture: &visuals.shengji_score_capture,
                    settlement: &visuals.shengji_settlement,
                    presentation: &visuals.shengji_presentation,
                },
            );
        } else if let Some(game) = client.0.model().uno_game() {
            render_uno_table(
                &mut commands,
                root,
                client,
                game,
                &ui,
                &chat,
                UnoTableVisuals {
                    assets: &visuals.ui,
                    avatars: &visuals.avatars,
                    appearance: &visuals.table,
                    brightness: form.table_brightness,
                    vignette: form.table_vignette,
                    table_materials: &mut table_materials,
                    turn_border_materials: &mut turn_border_materials,
                    game_summary: &visuals.game_summary,
                },
            );
        } else if let Some(game) = client.0.model().mahjong_game() {
            let interaction_menu_open = ui.social.interaction_menu_open;
            render_mahjong_table(
                &mut commands,
                root,
                client,
                game,
                &mut ui,
                &chat,
                interaction_menu_open,
                MahjongTableVisuals {
                    assets: &visuals.ui,
                    avatars: &visuals.avatars,
                    developer_hand: &developer_hand,
                    appearance: &visuals.table,
                    brightness: form.table_brightness,
                    vignette: form.table_vignette,
                    table_materials: &mut table_materials,
                    tile_materials: &mut mahjong_tile_materials,
                    game_summary: &visuals.game_summary,
                    claim_presentation: &visuals.mahjong_claim_presentation,
                },
            );
        } else {
            render_connection(
                &mut commands,
                root,
                &form,
                Some(&client.0),
                &visuals.ui,
                &visuals.avatars,
            );
        }
    } else {
        render_connection(
            &mut commands,
            root,
            &form,
            None,
            &visuals.ui,
            &visuals.avatars,
        );
    }
    if ui.navigation.settings_open {
        render_settings_modal(&mut commands, root, &form, &visuals.updater, &visuals.ui);
    }
    if ui.navigation.profile_open {
        let (name, avatar, reference_points, completed_games, game_profiles) =
            if let Some(player_profile) = ui.navigation.player_profile.as_ref() {
                (
                    player_profile.name.as_str(),
                    player_profile.avatar.as_ref(),
                    player_profile.reference_points,
                    player_profile.completed_games,
                    &player_profile.game_profiles,
                )
            } else {
                (
                    form.player_name.as_str(),
                    visuals.avatars.local.as_ref(),
                    profile.reference_points(),
                    profile.completed_games(),
                    profile.game_profiles(),
                )
            };
        render_profile_modal(
            &mut commands,
            root,
            name,
            avatar,
            reference_points,
            completed_games,
            game_profiles,
            ui.navigation.profile_game_tab,
            &visuals.ui,
        );
    }
    if ui.navigation.host_game_picker_open && client.is_none() {
        render_host_game_picker(&mut commands, root, &visuals.ui);
    }
    if visuals.updater.dialog_open {
        render_update_dialog(&mut commands, root, &visuals.updater, &visuals.ui);
    }
    if visuals.play_error.active
        && let Some(message) = visuals.play_error.message.as_deref()
    {
        add_play_error_popup(
            &mut commands,
            root,
            message,
            &visuals.play_error,
            &visuals.ui,
        );
    }
}
