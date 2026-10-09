//! 七鬼五二三牌桌、座位、手牌、出牌区、得分与计时布局。

use super::cards::{HandCardSpec, add_card_button};
use super::counter::render_counter;
use super::state::{ScoreCardsPopupPlacement, SeatVisuals};
use super::{
    NoLegalResponseHint, PlayEffectState, PlaySelectionCount, QIGUI_AVATAR_SIZE,
    QIGUI_PORTRAIT_WIDTH, QiGui523Assets, QiGui523UiAction, QiGui523UiState, QiGuiButtonTone,
    add_draw_pile, add_game_summary_modal, add_opponent_slot, add_play_effect_overlay,
    add_qigui_action_button, add_round_play, add_score_cards_popup, add_table_score_cards,
    game_has_legal_response, qigui_plate_image, sort_cards_high_to_low, spawn_round_play_container,
};
use crate::app::presentation::{
    GameSummaryAnimation, PlayerMenuProfile, PlayerPortraitSpec, StartGameSeatTransition, TEXT,
    TableBackground, TableBackgroundMaterial, TurnBorderAnimationKey, TurnBorderMaterial,
    add_auto_play_overlay, add_player_portrait, add_text, add_turn_border_trace_with_radius,
    attach_start_game_seat_transition, spawn_node, table_material_params,
};
use crate::app::runtime::{AvatarImages, ClientResource, TableAppearance, UiAssets};
#[cfg(feature = "developer")]
use crate::app::shell::add_developer_hand_input;
use crate::app::shell::{
    ChatPanelState, DeveloperHandInput, FinishedHandScoreSource, ScoreCaptureEffectState, SeatSide,
    SocialUiState, UiAction, add_chat_panel, add_reconnecting_overlay,
};
use bevy::prelude::*;
use leocard_client::NetworkState;
use leocard_protocol::{GameKind, GamePhaseView, QiGui523Snapshot, SeatId, TABLE_SEAT_COUNT};
use leocard_qigui523::QiGuiPlayKind;

pub(crate) struct TableVisualContext<'a> {
    pub assets: &'a UiAssets,
    pub game_assets: &'a QiGui523Assets,
    pub avatars: &'a AvatarImages,
    pub appearance: &'a TableAppearance,
    pub brightness: f32,
    pub vignette: f32,
    pub table_materials: &'a mut Assets<TableBackgroundMaterial>,
    pub game_summary: &'a GameSummaryAnimation,
    pub play_effect: &'a PlayEffectState,
    pub score_capture: &'a ScoreCaptureEffectState,
    pub start_game_transition: &'a StartGameSeatTransition,
    pub turn_border_materials: &'a mut Assets<TurnBorderMaterial>,
}

#[expect(
    clippy::too_many_arguments,
    reason = "the game-screen adapter passes common screen state plus grouped visuals"
)]
pub(crate) fn render_table(
    commands: &mut Commands,
    root: Entity,
    client: &ClientResource,
    game: &QiGui523Snapshot,
    ui: &QiGui523UiState,
    social: &SocialUiState,
    chat: &ChatPanelState,
    developer_hand: &DeveloperHandInput,
    visuals: &mut TableVisualContext,
) {
    #[cfg(not(feature = "developer"))]
    let _ = developer_hand;
    let TableVisualContext {
        assets,
        game_assets,
        avatars,
        appearance,
        brightness,
        vignette,
        table_materials,
        game_summary,
        play_effect,
        score_capture,
        start_game_transition,
        turn_border_materials,
    } = visuals;
    let content = spawn_node(
        commands,
        root,
        Node {
            width: percent(100),
            flex_grow: 1.0,
            min_height: px(0),
            position_type: PositionType::Relative,
            flex_direction: FlexDirection::Column,
            ..default()
        },
        None,
    );
    let felt = appearance
        .custom_felt
        .as_ref()
        .unwrap_or(&assets.table_felt)
        .clone();
    let material = table_materials.add(TableBackgroundMaterial {
        params: table_material_params(*brightness, *vignette, appearance.custom_felt.is_none()),
        texture: felt,
    });
    commands
        .entity(content)
        .insert((MaterialNode(material), TableBackground));
    let current = game.trick.as_ref().map(|trick| trick.current_player);
    let start_transition_active = start_game_transition.is_active_for(game.match_id);
    let table = spawn_node(
        commands,
        content,
        Node {
            width: percent(100),
            flex_grow: 1.0,
            min_height: px(410),
            position_type: PositionType::Relative,
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            ..default()
        },
        None,
    );
    if let NetworkState::Reconnecting(message) = client.0.state() {
        add_reconnecting_overlay(commands, table, message, assets);
    }
    let own_seat = game
        .players
        .iter()
        .find(|player| player.id == game.you)
        .expect("game snapshot contains the recipient")
        .seat;
    let seat_visuals = SeatVisuals {
        game,
        client,
        ui: assets,
        assets: game_assets,
        avatars,
        interaction_menu_open: social.interaction_menu_open,
        play_effect: play_effect.active.as_ref(),
        last_play: client.0.model().last_play_effect(),
        score_capture,
        start_transition_active,
        intro_only: start_transition_active,
    };
    for relative_seat in 1..TABLE_SEAT_COUNT {
        let physical_seat = SeatId((own_seat.0 + relative_seat) % TABLE_SEAT_COUNT);
        let player = game
            .players
            .iter()
            .find(|player| player.seat == physical_seat);
        add_opponent_slot(
            commands,
            table,
            relative_seat,
            player,
            player.is_some_and(|player| current == Some(player.id)),
            &seat_visuals,
            turn_border_materials,
        );
    }

    if start_transition_active {
        let hand_area = spawn_node(
            commands,
            content,
            Node {
                width: percent(100),
                height: px(218),
                flex_shrink: 0.0,
                position_type: PositionType::Relative,
                ..default()
            },
            None,
        );
        let own_seat = spawn_node(
            commands,
            hand_area,
            Node {
                position_type: PositionType::Absolute,
                left: px(10),
                bottom: px(8),
                width: px(QIGUI_PORTRAIT_WIDTH),
                height: px(76.0 * 1.17),
                ..default()
            },
            None,
        );
        if let Some(player) = game.players.iter().find(|player| player.id == game.you) {
            let portrait = add_player_portrait(
                commands,
                own_seat,
                Node {
                    width: px(QIGUI_PORTRAIT_WIDTH),
                    height: px(76.0 * 1.17),
                    ..default()
                },
                PlayerPortraitSpec {
                    player: player.id,
                    profile: PlayerMenuProfile {
                        name: &player.name,
                        avatar: player.avatar.and_then(|id| avatars.remote.get(&id)),
                        reference_points: player.reference_points,
                        completed_games: player.completed_games,
                        game_profiles: &player.game_profiles,
                    },
                    side: SeatSide::Left,
                    avatar_size: QIGUI_AVATAR_SIZE,
                    auto_play: player.auto_play,
                    menu_open: false,
                    menu_above: true,
                    name_color: TEXT,
                },
                assets,
            );
            attach_start_game_seat_transition(commands, portrait.portrait, game.you, true);
        }
        return;
    }

    render_counter(commands, table, ui, assets);
    let center = spawn_node(
        commands,
        table,
        Node {
            position_type: PositionType::Absolute,
            left: percent(42),
            top: percent(47),
            width: px(540),
            min_height: px(80),
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::FlexStart,
            ..default()
        },
        None,
    );
    let points_row = spawn_node(
        commands,
        center,
        Node {
            width: percent(100),
            min_height: px(80),
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::FlexStart,
            justify_content: JustifyContent::FlexStart,
            column_gap: px(18),
            ..default()
        },
        None,
    );
    add_draw_pile(commands, points_row, game.draw_pile_len.into(), assets);
    add_table_score_cards(commands, points_row, game, assets);
    let own_play = spawn_node(
        commands,
        table,
        Node {
            position_type: PositionType::Absolute,
            left: percent(27),
            right: percent(27),
            bottom: px(8),
            flex_direction: FlexDirection::Row,
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            column_gap: px(6),
            ..default()
        },
        None,
    );
    let own_cards = spawn_round_play_container(commands, own_play, JustifyContent::Center);
    add_round_play(
        commands,
        own_cards,
        game,
        game.you,
        play_effect.active.as_ref(),
        client.0.model().last_play_effect(),
        assets,
        game_assets,
    );

    let hand_area = spawn_node(
        commands,
        content,
        Node {
            width: percent(100),
            height: px(218),
            flex_shrink: 0.0,
            position_type: PositionType::Relative,
            padding: UiRect::all(px(16)),
            ..default()
        },
        None,
    );
    let self_state = game.players.iter().find(|player| player.id == game.you);

    let actions = spawn_node(
        commands,
        hand_area,
        Node {
            position_type: PositionType::Absolute,
            left: percent(30),
            right: percent(30),
            top: px(-16),
            height: px(58),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            column_gap: px(8),
            ..default()
        },
        None,
    );
    match &game.phase {
        GamePhaseView::Playing if current == Some(game.you) => {
            let is_leading = game
                .trick
                .as_ref()
                .is_some_and(|trick| trick.winning_play.is_none());
            let can_follow = is_leading
                || client
                    .0
                    .model()
                    .qigui523_rules()
                    .is_some_and(|rules| game_has_legal_response(game, rules));
            if can_follow {
                let (_, selection_label) = add_qigui_action_button(
                    commands,
                    actions,
                    &format!("出牌 ({})", ui.selected.len()),
                    UiAction::QiGui523(QiGui523UiAction::Play),
                    QiGuiButtonTone::Play,
                    assets,
                );
                commands.entity(selection_label).insert(PlaySelectionCount);
            }
            if !is_leading && can_follow {
                add_qigui_action_button(
                    commands,
                    actions,
                    "不要",
                    UiAction::QiGui523(QiGui523UiAction::Pass),
                    QiGuiButtonTone::Pass,
                    assets,
                );
                add_qigui_action_button(
                    commands,
                    actions,
                    "提示",
                    UiAction::QiGui523(QiGui523UiAction::Hint),
                    QiGuiButtonTone::Hint,
                    assets,
                );
            } else if !is_leading {
                let hint = spawn_node(
                    commands,
                    actions,
                    Node {
                        position_type: PositionType::Absolute,
                        left: px(0),
                        right: px(0),
                        bottom: px(61),
                        height: px(38),
                        align_items: AlignItems::Center,
                        justify_content: JustifyContent::Center,
                        ..default()
                    },
                    None,
                );
                commands.entity(hint).insert((
                    NoLegalResponseHint,
                    UiTransform::IDENTITY,
                    GlobalZIndex(900),
                ));
                let hint_plate = spawn_node(
                    commands,
                    hint,
                    Node {
                        width: px(280),
                        height: px(38),
                        align_items: AlignItems::Center,
                        justify_content: JustifyContent::Center,
                        ..default()
                    },
                    None,
                );
                commands
                    .entity(hint_plate)
                    .insert(qigui_plate_image(assets));
                add_text(commands, hint_plate, "没有牌能大过上家", 15.0, TEXT, assets);
                add_qigui_action_button(
                    commands,
                    actions,
                    "不要",
                    UiAction::QiGui523(QiGui523UiAction::Pass),
                    QiGuiButtonTone::Pass,
                    assets,
                );
            }
        }
        GamePhaseView::Playing => {}
        GamePhaseView::Finished { .. } => {}
    }

    let hand = spawn_node(
        commands,
        hand_area,
        Node {
            position_type: PositionType::Absolute,
            left: px(0),
            right: px(0),
            bottom: px(0),
            height: px(134),
            flex_direction: FlexDirection::Row,
            flex_wrap: FlexWrap::NoWrap,
            justify_content: JustifyContent::Center,
            align_items: AlignItems::FlexEnd,
            ..default()
        },
        None,
    );
    if matches!(game.phase, GamePhaseView::Finished { .. }) {
        commands
            .entity(hand)
            .insert(FinishedHandScoreSource(game.you));
    }
    let mut displayed_hand = game.your_hand.clone();
    sort_cards_high_to_low(&mut displayed_hand);
    let last_card = displayed_hand.len().saturating_sub(1);
    for (index, card) in displayed_hand.iter().enumerate() {
        let animation = ui.card_animations.get(card).copied().unwrap_or_default();
        add_card_button(
            commands,
            hand,
            HandCardSpec {
                card: *card,
                selected: ui.selected.contains(card),
                animation,
                index,
                hand_len: displayed_hand.len(),
                is_last: index == last_card,
            },
            assets,
        );
    }

    let own_portrait_height = 76.0 * 1.17;
    let own_seat = spawn_node(
        commands,
        hand_area,
        Node {
            position_type: PositionType::Absolute,
            left: px(10),
            bottom: px(8),
            width: px(QIGUI_PORTRAIT_WIDTH),
            height: px(own_portrait_height),
            ..default()
        },
        None,
    );
    if let Some(player) = self_state {
        let portrait = add_player_portrait(
            commands,
            own_seat,
            Node {
                width: px(QIGUI_PORTRAIT_WIDTH),
                height: px(own_portrait_height),
                ..default()
            },
            PlayerPortraitSpec {
                player: player.id,
                profile: PlayerMenuProfile {
                    name: &player.name,
                    avatar: player.avatar.and_then(|id| avatars.remote.get(&id)),
                    reference_points: player.reference_points,
                    completed_games: player.completed_games,
                    game_profiles: &player.game_profiles,
                },
                side: SeatSide::Left,
                avatar_size: QIGUI_AVATAR_SIZE,
                auto_play: player.auto_play,
                menu_open: social.interaction_menu_open == Some(player.id),
                menu_above: true,
                name_color: TEXT,
            },
            assets,
        );
        attach_start_game_seat_transition(
            commands,
            portrait.portrait,
            game.you,
            start_transition_active,
        );
        if current == Some(game.you) {
            add_turn_border_trace_with_radius(
                commands,
                portrait.avatar_ring,
                turn_border_materials,
                TurnBorderAnimationKey::new(GameKind::QiGui523, game.match_id, game.you),
                QIGUI_AVATAR_SIZE * 0.2,
                QIGUI_AVATAR_SIZE,
            );
        }
    } else {
        add_text(commands, own_seat, "你", 13.0, TEXT, assets);
    }
    #[cfg(feature = "developer")]
    if matches!(game.phase, GamePhaseView::Playing) {
        add_developer_hand_input(
            commands,
            hand_area,
            developer_hand,
            "编辑手牌，如 70523",
            Vec2::new(10.0, own_portrait_height + 16.0),
            assets,
        );
    }
    let local_auto_play = matches!(game.phase, GamePhaseView::Playing)
        .then(|| self_state.is_some_and(|player| player.auto_play));
    add_chat_panel(commands, content, chat, assets, local_auto_play, &[]);
    if local_auto_play == Some(true) {
        add_auto_play_overlay(commands, content, assets);
    }
    if let Some(player) = self_state {
        add_score_cards_popup(
            commands,
            hand_area,
            player,
            client.0.model().captured_score_cards(player.id),
            ScoreCardsPopupPlacement::Own,
            score_capture,
            assets,
        );
    }
    if matches!(game.phase, GamePhaseView::Finished { .. }) {
        add_game_summary_modal(commands, table, game, assets, avatars, game_summary);
    }
    if let Some(effect) = play_effect.active.as_ref().filter(|effect| {
        matches!(
            effect.play.kind,
            QiGuiPlayKind::Bomb(_) | QiGuiPlayKind::HeavenBomb
        )
    }) {
        add_play_effect_overlay(commands, table, root, game, effect, assets);
    }
}
