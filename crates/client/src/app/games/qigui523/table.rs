//! 七鬼五二三牌桌、座位、手牌、出牌区、得分与计时布局。

use super::*;
use leocard_client::NetworkState;
use leocard_protocol::QiGui523Snapshot;
use leocard_protocol::{GameKind, GamePhaseView, SeatId, TABLE_SEAT_COUNT};
use leocard_qigui523::QiGuiPlayKind;

pub fn lobby_seat_position(seat: u8) -> (f32, f32) {
    match seat {
        0 => (150.0, 290.0),
        1 => (0.0, 145.0),
        2 => (150.0, 0.0),
        3 => (328.0, 0.0),
        4 => (478.0, 145.0),
        5 => (328.0, 290.0),
        _ => unreachable!("there are exactly six table seats"),
    }
}

pub struct TableVisualContext<'a> {
    pub assets: &'a UiAssets,
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

pub fn render_table(
    commands: &mut Commands,
    root: Entity,
    client: &ClientResource,
    game: &QiGui523Snapshot,
    ui: &UiState,
    chat: &ChatPanelState,
    developer_hand: &DeveloperHandInput,
    visuals: &mut TableVisualContext,
) {
    #[cfg(not(feature = "developer"))]
    let _ = developer_hand;
    let TableVisualContext {
        assets,
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
        avatars,
        interaction_menu_open: ui.social.interaction_menu_open,
        play_effect: play_effect.active.as_ref(),
        last_play: client.0.model().last_play_effect(),
        score_capture,
        start_transition_active,
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

    let center = spawn_node(
        commands,
        table,
        Node {
            position_type: PositionType::Absolute,
            left: percent(32),
            right: percent(32),
            top: percent(42),
            min_height: px(142),
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            row_gap: px(3),
            ..default()
        },
        None,
    );
    let table_points = game.trick.as_ref().map_or(0, |trick| trick.table_points);
    add_draw_pile(commands, center, game.draw_pile_len.into(), assets);
    let points_row = spawn_node(
        commands,
        center,
        Node {
            min_height: px(CardSize::TableScore.dimensions().1),
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            column_gap: px(7),
            ..default()
        },
        None,
    );
    add_text(
        commands,
        points_row,
        format!("桌面  {table_points} 分"),
        16.0,
        TEXT,
        assets,
    );
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
    );

    let hand_area = spawn_node(
        commands,
        content,
        Node {
            width: percent(100),
            height: px(180),
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
            top: px(2),
            height: px(48),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            column_gap: px(12),
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
                let (_, selection_label) = add_action_button_with_label(
                    commands,
                    actions,
                    &format!("出牌 ({})", ui.qigui523.selected.len()),
                    UiAction::Play,
                    ButtonKind::Primary,
                    assets,
                );
                commands.entity(selection_label).insert(PlaySelectionCount);
            }
            if !is_leading && can_follow {
                add_action_button(
                    commands,
                    actions,
                    "不要",
                    UiAction::Pass,
                    ButtonKind::Pass,
                    assets,
                );
                add_action_button(
                    commands,
                    actions,
                    "提示",
                    UiAction::Hint,
                    ButtonKind::Secondary,
                    assets,
                );
            } else if !is_leading {
                let hint = spawn_node(
                    commands,
                    actions,
                    Node {
                        position_type: PositionType::Absolute,
                        left: px(-80),
                        right: px(-80),
                        bottom: px(50),
                        height: px(28),
                        align_items: AlignItems::Center,
                        justify_content: JustifyContent::Center,
                        border_radius: BorderRadius::all(px(7)),
                        ..default()
                    },
                    Some(Color::BLACK.with_alpha(0.58)),
                );
                commands.entity(hint).insert((
                    NoLegalResponseHint,
                    UiTransform::IDENTITY,
                    GlobalZIndex(900),
                ));
                add_text(commands, hint, "没有牌能大过上家", 17.0, ACCENT, assets);
                add_action_button(
                    commands,
                    actions,
                    "不要",
                    UiAction::Pass,
                    ButtonKind::Pass,
                    assets,
                );
            }
        }
        GamePhaseView::Playing => {
            add_text(commands, actions, "等待其他玩家出牌…", 15.0, MUTED, assets);
        }
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
            height: px(105),
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
        let animation = ui
            .qigui523
            .card_animations
            .get(card)
            .copied()
            .unwrap_or_default();
        add_card_button(
            commands,
            hand,
            HandCardSpec {
                card: *card,
                selected: ui.qigui523.selected.contains(card),
                animation,
                index,
                hand_len: displayed_hand.len(),
                is_last: index == last_card,
            },
            assets,
        );
    }

    let self_summary = spawn_node(
        commands,
        hand_area,
        Node {
            position_type: PositionType::Absolute,
            left: px(10),
            bottom: px(8),
            width: px(118),
            min_width: px(118),
            max_width: px(118),
            height: px(48),
            min_height: px(48),
            max_height: px(48),
            padding: UiRect::axes(px(3), px(2)),
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            column_gap: px(2),
            border: UiRect::all(px(1)),
            border_radius: BorderRadius::all(px(7)),
            ..default()
        },
        Some(HEADER_BG.with_alpha(0.92)),
    );
    commands
        .entity(self_summary)
        .insert(BorderColor::all(BORDER));
    attach_start_game_seat_transition(commands, self_summary, game.you, start_transition_active);
    decorate_player_panel(commands, self_summary, assets, 0.72);
    if current == Some(game.you) {
        add_turn_border_trace(
            commands,
            self_summary,
            turn_border_materials,
            TurnBorderAnimationKey::new(GameKind::QiGui523, game.match_id, game.you),
        );
    }
    if let Some(player) = self_state {
        let handle = player.avatar.and_then(|id| avatars.remote.get(&id));
        let avatar = add_avatar(commands, self_summary, &player.name, handle, 24.0, assets);
        commands
            .entity(avatar)
            .insert(PlayerAvatarAnchor(player.id));
        let details = spawn_node(
            commands,
            self_summary,
            Node {
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                ..default()
            },
            None,
        );
        add_text(commands, details, &player.name, 11.0, TEXT, assets);
        add_text(
            commands,
            details,
            reference_points_label(player.reference_points),
            7.5,
            ACCENT,
            assets,
        );
    } else {
        add_text(commands, self_summary, "你", 13.0, TEXT, assets);
    }
    #[cfg(feature = "developer")]
    if matches!(game.phase, GamePhaseView::Playing) {
        add_developer_hand_input(
            commands,
            hand_area,
            developer_hand,
            "编辑手牌，如 70523",
            Vec2::new(136.0, 8.0),
            assets,
        );
    }
    let local_auto_play = matches!(game.phase, GamePhaseView::Playing)
        .then(|| self_state.is_some_and(|player| player.auto_play));
    add_chat_panel(commands, content, chat, assets, local_auto_play, None, None);
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
