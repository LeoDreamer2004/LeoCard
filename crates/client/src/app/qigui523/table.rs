//! 七鬼五二三牌桌、座位、手牌、出牌区、得分与计时布局。

use super::*;

pub(in crate::app) fn lobby_seat_position(seat: u8) -> (f32, f32) {
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

pub(in crate::app) struct TableVisualContext<'a> {
    pub(in crate::app) assets: &'a UiAssets,
    pub(in crate::app) avatars: &'a AvatarImages,
    pub(in crate::app) appearance: &'a TableAppearance,
    pub(in crate::app) brightness: f32,
    pub(in crate::app) vignette: f32,
    pub(in crate::app) table_materials: &'a mut Assets<TableBackgroundMaterial>,
    pub(in crate::app) game_summary: &'a GameSummaryAnimation,
    pub(in crate::app) play_effect: &'a PlayEffectState,
    pub(in crate::app) score_capture: &'a ScoreCaptureEffectState,
    pub(in crate::app) start_game_transition: &'a StartGameSeatTransition,
    pub(in crate::app) turn_border_materials: &'a mut Assets<TurnBorderMaterial>,
}

pub(in crate::app) fn render_table(
    commands: &mut Commands,
    root: Entity,
    client: &ClientResource,
    game: &leocard_protocol::QiGui523Snapshot,
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
    let tiled = appearance.custom_felt.is_none();
    let material = table_materials.add(TableBackgroundMaterial {
        params: table_material_params(*brightness, *vignette, tiled),
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
        interaction_menu_open: ui.interaction_menu_open,
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
                let selection_label = add_action_button(
                    commands,
                    actions,
                    &format!("出牌 ({})", ui.selected.len()),
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
    commands.entity(self_summary).insert((
        BorderColor::all(BORDER),
        GameSeatTransitionTarget(game.you),
        GameSeatTransitionPose::default(),
        UiTransform::IDENTITY,
    ));
    if start_transition_active {
        commands.entity(self_summary).insert(Visibility::Hidden);
    }
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
        add_developer_hand_input(commands, hand_area, developer_hand, assets);
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
    if let Some(effect) = play_effect
        .active
        .as_ref()
        .filter(|effect| matches!(effect.play.kind, PlayKind::Bomb(_) | PlayKind::HeavenBomb))
    {
        add_play_effect_overlay(commands, table, root, game, effect, assets);
    }
}

fn add_draw_pile(commands: &mut Commands, parent: Entity, count: usize, assets: &UiAssets) {
    let layers = if count == 0 {
        0
    } else {
        count.div_ceil(14).clamp(1, 8)
    };
    let pile = spawn_node(
        commands,
        parent,
        Node {
            width: px(48.0 + layers as f32 * 2.0),
            height: px(64.0 + layers as f32 * 1.5),
            position_type: PositionType::Relative,
            margin: UiRect::bottom(px(3)),
            ..default()
        },
        None,
    );
    if layers == 0 {
        let empty = spawn_node(
            commands,
            pile,
            Node {
                position_type: PositionType::Absolute,
                left: px(1),
                top: px(1),
                width: px(46),
                height: px(62),
                border: UiRect::all(px(2)),
                border_radius: BorderRadius::all(px(5)),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
            Some(HEADER_BG.with_alpha(0.36)),
        );
        commands
            .entity(empty)
            .insert(BorderColor::all(MUTED.with_alpha(0.45)));
    } else {
        for layer in 0..layers {
            let card = commands
                .spawn((
                    Node {
                        position_type: PositionType::Absolute,
                        left: px(layer as f32 * 2.0),
                        top: px((layers - layer - 1) as f32 * 1.5),
                        width: px(46),
                        height: px(62),
                        border: UiRect::all(px(1)),
                        border_radius: BorderRadius::all(px(5)),
                        ..default()
                    },
                    ImageNode::new(assets.card_back.clone()),
                    BorderColor::all(TEXT.with_alpha(0.55)),
                    BoxShadow::new(Color::BLACK.with_alpha(0.28), px(1), px(2), px(0), px(3)),
                    ZIndex(layer as i32),
                ))
                .id();
            commands.entity(pile).add_child(card);
        }
    }

    let counter = spawn_node(
        commands,
        pile,
        Node {
            position_type: PositionType::Absolute,
            left: percent(50),
            top: percent(50),
            width: px(34),
            height: px(28),
            margin: UiRect {
                left: px(-17),
                top: px(-14),
                ..default()
            },
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            border_radius: BorderRadius::all(px(7)),
            ..default()
        },
        Some(Color::BLACK.with_alpha(0.72)),
    );
    commands.entity(counter).insert(ZIndex(layers as i32 + 1));
    add_text(commands, counter, count.to_string(), 18.0, TEXT, assets);
}

fn add_table_score_cards(
    commands: &mut Commands,
    parent: Entity,
    game: &leocard_protocol::QiGui523Snapshot,
    assets: &UiAssets,
) {
    let Some(trick) = game.trick.as_ref() else {
        return;
    };
    let mut cards = trick
        .records
        .iter()
        .flat_map(|record| match record {
            PublicPlayRecord::Played { play, .. } => play.cards.as_slice(),
            PublicPlayRecord::Passed { .. } => &[],
        })
        .copied()
        .filter(|card| card.score() > 0)
        .collect::<Vec<_>>();
    if cards.is_empty() {
        return;
    }
    sort_cards_high_to_low(&mut cards);
    let hand = spawn_node(
        commands,
        parent,
        Node {
            height: px(CardSize::TableScore.dimensions().1),
            flex_direction: FlexDirection::Row,
            flex_wrap: FlexWrap::NoWrap,
            align_items: AlignItems::Center,
            ..default()
        },
        None,
    );
    let last_card = cards.len().saturating_sub(1);
    for (index, card) in cards.into_iter().enumerate() {
        add_card_image(
            commands,
            hand,
            card,
            CardSize::TableScore,
            index,
            index == last_card,
            false,
            assets,
        );
    }
}

struct SeatVisuals<'a> {
    game: &'a leocard_protocol::QiGui523Snapshot,
    client: &'a ClientResource,
    ui: &'a UiAssets,
    avatars: &'a AvatarImages,
    interaction_menu_open: Option<PlayerId>,
    play_effect: Option<&'a ActivePlayEffect>,
    last_play: Option<&'a (PlayerId, PublicPlay)>,
    score_capture: &'a ScoreCaptureEffectState,
    start_transition_active: bool,
}

fn add_opponent_slot(
    commands: &mut Commands,
    table: Entity,
    relative_seat: u8,
    player: Option<&PlayerPublicState>,
    active: bool,
    visuals: &SeatVisuals,
    turn_border_materials: &mut Assets<TurnBorderMaterial>,
) {
    const SIDE_PLAY_GAP: f32 = 52.0;
    const SIDE_SLOT_WIDTH: f32 = 224.0 + SIDE_PLAY_GAP + 148.0;
    let side = match relative_seat {
        1 | 2 => SeatSide::Left,
        3 => SeatSide::Top,
        4 | 5 => SeatSide::Right,
        _ => unreachable!("the local player occupies relative seat zero"),
    };
    let mut node = Node {
        position_type: PositionType::Absolute,
        min_height: px(if matches!(side, SeatSide::Top) {
            140
        } else {
            100
        }),
        align_items: AlignItems::Center,
        justify_content: match side {
            SeatSide::Left => JustifyContent::FlexStart,
            SeatSide::Top => JustifyContent::Center,
            SeatSide::Right => JustifyContent::FlexEnd,
        },
        // 左右玩家的机器人标记会伸出玩家框 38px；为出牌区预留独立间距，
        // 避免牌组覆盖标记及其天线动画。
        column_gap: px(if matches!(side, SeatSide::Top) {
            8.0
        } else {
            SIDE_PLAY_GAP
        }),
        row_gap: px(5),
        ..default()
    };
    match relative_seat {
        1 => {
            node.left = px(10);
            node.bottom = px(36);
            node.width = px(SIDE_SLOT_WIDTH);
        }
        2 => {
            node.left = px(10);
            node.top = px(52);
            node.width = px(SIDE_SLOT_WIDTH);
        }
        3 => {
            node.left = percent(32);
            node.right = percent(32);
            node.top = px(4);
            node.flex_direction = FlexDirection::Column;
        }
        4 => {
            node.right = px(10);
            node.top = px(52);
            node.width = px(SIDE_SLOT_WIDTH);
        }
        5 => {
            node.right = px(10);
            node.bottom = px(36);
            node.width = px(SIDE_SLOT_WIDTH);
        }
        _ => unreachable!(),
    }
    let slot = spawn_node(commands, table, node, None);
    if matches!(side, SeatSide::Right) {
        add_round_play_for_optional_player(
            commands,
            slot,
            side,
            visuals.game,
            player,
            visuals.play_effect,
            visuals.last_play,
            visuals.ui,
        );
    }
    let badge = spawn_node(
        commands,
        slot,
        Node {
            width: px(224),
            min_width: px(224),
            height: px(72),
            min_height: px(72),
            max_height: px(72),
            flex_shrink: 0.0,
            align_self: AlignSelf::Center,
            padding: if player.is_none() {
                UiRect::axes(px(8), px(4))
            } else {
                match side {
                    SeatSide::Left => UiRect::new(px(8), px(68), px(4), px(4)),
                    SeatSide::Right => UiRect::new(px(68), px(8), px(4), px(4)),
                    SeatSide::Top => UiRect::new(px(8), px(68), px(4), px(4)),
                }
            },
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            column_gap: px(6),
            border: UiRect::all(px(1)),
            border_radius: BorderRadius::all(px(8)),
            ..default()
        },
        Some(if active { PANEL_ALT } else { HEADER_BG }),
    );
    let interaction_menu_open =
        player.is_some_and(|player| visuals.interaction_menu_open == Some(player.id));
    commands.entity(badge).insert(BorderColor::all(BORDER));
    decorate_player_panel(commands, badge, visuals.ui, 1.0);
    if active && let Some(player) = player {
        add_turn_border_trace(
            commands,
            badge,
            turn_border_materials,
            TurnBorderAnimationKey::new(GameKind::QiGui523, visuals.game.match_id, player.id),
        );
    }
    match player {
        Some(player) => {
            commands.entity(badge).insert((
                Button,
                UiAction::ToggleInteractionMenu(player.id),
                GameSeatTransitionTarget(player.id),
                GameSeatTransitionPose::default(),
                UiTransform::IDENTITY,
            ));
            if visuals.start_transition_active {
                commands.entity(badge).insert(Visibility::Hidden);
            }
            let handle = player.avatar.and_then(|id| visuals.avatars.remote.get(&id));
            let avatar = add_avatar(commands, badge, &player.name, handle, 32.0, visuals.ui);
            commands
                .entity(avatar)
                .insert(PlayerAvatarAnchor(player.id));
            if player.auto_play {
                add_auto_play_robot_indicator(commands, badge, player.id, side, visuals.ui);
            }
            let details = spawn_node(
                commands,
                badge,
                Node {
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    ..default()
                },
                None,
            );
            add_text(
                commands,
                details,
                format!(
                    "{}{}",
                    player.name,
                    if player.connected { "" } else { " [离线]" }
                ),
                14.0,
                if active { ACCENT } else { TEXT },
                visuals.ui,
            );
            add_text(
                commands,
                details,
                reference_points_label(player.reference_points),
                11.0,
                ACCENT,
                visuals.ui,
            );
            add_text(
                commands,
                details,
                format!("剩余 {} 张", player.hand_len),
                12.0,
                MUTED,
                visuals.ui,
            );
            let score = displayed_captured_score(visuals.score_capture, player.id, player.score);
            let score_text =
                add_player_panel_primary_value(commands, badge, side, score, visuals.ui);
            commands
                .entity(score_text)
                .insert(PlayerGameScoreText::Opponent {
                    player: player.id,
                    side,
                });
            if let Some(cards) = finished_remaining_hand(visuals.game, player.id)
                && !cards.is_empty()
            {
                add_finished_remaining_hand(commands, badge, player.id, cards, visuals.ui);
            }
            let score_popup = add_score_cards_popup(
                commands,
                badge,
                player,
                visuals.client.0.model().captured_score_cards(player.id),
                ScoreCardsPopupPlacement::Opponent(side),
                visuals.score_capture,
                visuals.ui,
            );
            commands.entity(score_popup).insert(Visibility::Hidden);
            let interaction_menu = add_interaction_menu(
                commands,
                badge,
                player.id,
                side,
                &player.name,
                handle,
                player.reference_points,
                player.completed_games,
                visuals.ui,
            );
            commands
                .entity(interaction_menu)
                .insert(if interaction_menu_open {
                    Visibility::Visible
                } else {
                    Visibility::Hidden
                });
            commands.entity(badge).insert(OpponentBadge {
                player: player.id,
                score_popup: Some(score_popup),
                interaction_menu,
            });
        }
        None => {
            add_text(commands, badge, "空位", 14.0, MUTED, visuals.ui);
        }
    }
    if !matches!(side, SeatSide::Right) {
        add_round_play_for_optional_player(
            commands,
            slot,
            side,
            visuals.game,
            player,
            visuals.play_effect,
            visuals.last_play,
            visuals.ui,
        );
    }
}

pub(in crate::app) fn add_auto_play_robot_indicator(
    commands: &mut Commands,
    badge: Entity,
    player: PlayerId,
    side: SeatSide,
    assets: &UiAssets,
) {
    let mut node = Node {
        position_type: PositionType::Absolute,
        top: px(19),
        width: px(34),
        height: px(34),
        ..default()
    };
    match side {
        SeatSide::Left | SeatSide::Top => node.right = px(-38),
        SeatSide::Right => node.left = px(-38),
    }
    let indicator = commands
        .spawn((
            node,
            ImageNode::new(assets.robot_icon.clone()),
            UiTransform::IDENTITY,
            ZIndex(30),
            FocusPolicy::Pass,
            AutoPlayRobotIndicator,
        ))
        .id();
    commands.entity(badge).add_child(indicator);
    add_auto_play_antenna_lights(commands, indicator, player);
}

fn add_auto_play_antenna_lights(commands: &mut Commands, indicator: Entity, player: PlayerId) {
    let glow = commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                left: px(10.5),
                top: px(-3),
                width: px(13),
                height: px(13),
                border_radius: BorderRadius::all(percent(50)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.32, 1.0, 0.58, 0.0)),
            UiTransform::IDENTITY,
            ZIndex(2),
            FocusPolicy::Pass,
            AutoPlayAntennaLight {
                player,
                part: AutoPlayAntennaLightPart::Glow,
            },
        ))
        .id();
    commands.entity(indicator).add_child(glow);

    for (left, top, rotation) in [(16.0, -8.0, 0.0), (7.5, -4.5, -0.82), (24.5, -4.5, 0.82)] {
        let ray = commands
            .spawn((
                Node {
                    position_type: PositionType::Absolute,
                    left: px(left),
                    top: px(top),
                    width: px(2),
                    height: px(6),
                    border_radius: BorderRadius::all(px(1)),
                    ..default()
                },
                BackgroundColor(Color::srgba(0.46, 1.0, 0.68, 0.0)),
                UiTransform {
                    rotation: Rot2::radians(rotation),
                    ..UiTransform::IDENTITY
                },
                ZIndex(3),
                FocusPolicy::Pass,
                AutoPlayAntennaLight {
                    player,
                    part: AutoPlayAntennaLightPart::Ray,
                },
            ))
            .id();
        commands.entity(indicator).add_child(ray);
    }
}

fn finished_remaining_hand(
    game: &leocard_protocol::QiGui523Snapshot,
    player: PlayerId,
) -> Option<&[Card]> {
    let GamePhaseView::Finished {
        remaining_hands, ..
    } = &game.phase
    else {
        return None;
    };
    remaining_hands
        .iter()
        .find(|hand| hand.player == player)
        .map(|hand| hand.cards.as_slice())
}

fn add_finished_remaining_hand(
    commands: &mut Commands,
    badge: Entity,
    player: PlayerId,
    cards: &[Card],
    assets: &UiAssets,
) {
    let hand = spawn_node(
        commands,
        badge,
        Node {
            position_type: PositionType::Absolute,
            left: px(0),
            top: px(74),
            width: percent(100),
            height: px(CardSize::FinishedHand.dimensions().1),
            flex_direction: FlexDirection::Row,
            flex_wrap: FlexWrap::NoWrap,
            align_items: AlignItems::FlexStart,
            justify_content: JustifyContent::Center,
            ..default()
        },
        None,
    );
    commands.entity(hand).insert((
        FinishedHandScoreSource(player),
        GlobalZIndex(850),
        FocusPolicy::Pass,
    ));
    let mut displayed_cards = cards.to_vec();
    sort_cards_high_to_low(&mut displayed_cards);
    let last_card = displayed_cards.len().saturating_sub(1);
    for (index, card) in displayed_cards.into_iter().enumerate() {
        add_card_image(
            commands,
            hand,
            card,
            CardSize::FinishedHand,
            index,
            index == last_card,
            false,
            assets,
        );
    }
}

#[derive(Clone, Copy)]
enum ScoreCardsPopupPlacement {
    Opponent(SeatSide),
    Own,
}

pub(in crate::app) fn add_interaction_menu(
    commands: &mut Commands,
    parent: Entity,
    target: PlayerId,
    side: SeatSide,
    player_name: &str,
    avatar: Option<&Handle<Image>>,
    reference_points: i32,
    completed_games: u32,
    assets: &UiAssets,
) -> Entity {
    let mut node = Node {
        position_type: PositionType::Absolute,
        width: px(330),
        min_height: px(118),
        padding: UiRect::all(px(6)),
        flex_direction: FlexDirection::Column,
        align_items: AlignItems::Stretch,
        row_gap: px(4),
        border: UiRect::all(px(1)),
        border_radius: BorderRadius::all(px(8)),
        ..default()
    };
    position_opponent_popup(&mut node, side);
    let menu = spawn_node(commands, parent, node, Some(HEADER_BG.with_alpha(0.98)));
    commands.entity(menu).insert((
        InteractionMenuPanel(target),
        BorderColor::all(ACCENT.with_alpha(0.72)),
        GlobalZIndex(1500),
        FocusPolicy::Pass,
    ));
    let profile = spawn_node(
        commands,
        menu,
        Node {
            width: percent(100),
            height: px(38),
            min_height: px(38),
            padding: UiRect::axes(px(5), px(3)),
            align_items: AlignItems::Center,
            column_gap: px(7),
            border_radius: BorderRadius::all(px(6)),
            ..default()
        },
        Some(PANEL_ALT.with_alpha(0.82)),
    );
    add_avatar(commands, profile, player_name, avatar, 30.0, assets);
    let identity = spawn_node(
        commands,
        profile,
        Node {
            min_width: px(0),
            flex_grow: 1.0,
            flex_direction: FlexDirection::Column,
            justify_content: JustifyContent::Center,
            row_gap: px(1),
            ..default()
        },
        None,
    );
    add_text(commands, identity, player_name, 13.0, TEXT, assets);
    add_text(
        commands,
        identity,
        format!(
            "等级:{}  分数:{}  对局:{}",
            reference_level(reference_points),
            reference_points,
            completed_games
        ),
        9.5,
        MUTED,
        assets,
    );
    let actions = spawn_node(
        commands,
        menu,
        Node {
            width: percent(100),
            height: px(62),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::SpaceEvenly,
            column_gap: px(5),
            ..default()
        },
        None,
    );
    for (kind, label) in [
        (PlayerInteractionKind::Flower, "鲜花"),
        (PlayerInteractionKind::Egg, "鸡蛋"),
        (PlayerInteractionKind::Wine, "酒杯"),
        (PlayerInteractionKind::Shoe, "拖鞋"),
    ] {
        let normal = Color::srgb(0.18, 0.42, 0.34);
        let button = commands
            .spawn((
                Button,
                UiAction::SendInteraction { target, kind },
                ButtonTint {
                    normal,
                    hovered: Color::srgb(0.27, 0.58, 0.46),
                    pressed: Color::srgb(0.12, 0.30, 0.24),
                },
                Node {
                    width: px(72),
                    height: px(62),
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    row_gap: px(1),
                    border_radius: BorderRadius::all(px(6)),
                    ..default()
                },
                ImageNode::new(assets.secondary_button.clone())
                    .with_mode(NodeImageMode::Stretch)
                    .with_color(normal),
            ))
            .id();
        commands.entity(actions).add_child(button);
        let icon = commands
            .spawn((
                Node {
                    width: px(38),
                    height: px(38),
                    ..default()
                },
                ImageNode::new(
                    assets
                        .interaction_images
                        .get(&(kind, false))
                        .expect("every interaction has a flight image")
                        .clone(),
                ),
                FocusPolicy::Pass,
            ))
            .id();
        commands.entity(button).add_child(icon);
        add_text(commands, button, label, 11.0, TEXT, assets);
        if matches!(
            kind,
            PlayerInteractionKind::Wine | PlayerInteractionKind::Shoe
        ) {
            let mask = commands
                .spawn((
                    Node {
                        position_type: PositionType::Absolute,
                        left: px(0),
                        right: px(0),
                        top: px(0),
                        bottom: px(0),
                        border_radius: BorderRadius::all(px(6)),
                        ..default()
                    },
                    ImageNode::new(
                        assets
                            .interaction_cooldown_masks
                            .last()
                            .cloned()
                            .unwrap_or_default(),
                    ),
                    InteractionCooldownMask {
                        player: target,
                        kind,
                    },
                    Visibility::Hidden,
                    ZIndex(10),
                    FocusPolicy::Pass,
                ))
                .id();
            commands.entity(button).add_child(mask);
        }
    }
    menu
}

pub(in crate::app) fn position_opponent_popup(node: &mut Node, side: SeatSide) {
    match side {
        SeatSide::Left => {
            node.left = px(0);
            node.top = px(76);
        }
        SeatSide::Top => {
            node.left = px(-81);
            node.top = px(76);
        }
        SeatSide::Right => {
            node.right = px(0);
            node.top = px(76);
        }
    }
}

fn add_score_cards_popup(
    commands: &mut Commands,
    parent: Entity,
    player: &PlayerPublicState,
    cards: &[Card],
    placement: ScoreCardsPopupPlacement,
    score_capture: &ScoreCaptureEffectState,
    assets: &UiAssets,
) -> Entity {
    let mut node = Node {
        position_type: PositionType::Absolute,
        width: px(330),
        padding: UiRect::all(px(6)),
        flex_direction: FlexDirection::Column,
        align_items: AlignItems::Stretch,
        row_gap: px(3),
        border: UiRect::all(px(1)),
        border_radius: BorderRadius::all(px(8)),
        ..default()
    };
    match placement {
        ScoreCardsPopupPlacement::Opponent(side) => position_opponent_popup(&mut node, side),
        ScoreCardsPopupPlacement::Own => {
            node.left = px(10);
            node.bottom = px(64);
            node.width = px(410);
            node.min_height = px(58);
            node.padding = UiRect::new(px(6), px(76), px(6), px(6));
        }
    }
    let popup = spawn_node(commands, parent, node, Some(Color::BLACK.with_alpha(0.30)));
    commands.entity(popup).insert((
        BorderColor::all(ACCENT.with_alpha(0.72)),
        GlobalZIndex(1500),
        FocusPolicy::Pass,
    ));
    let displayed_score = displayed_captured_score(score_capture, player.id, player.score);
    match placement {
        ScoreCardsPopupPlacement::Own => {
            let score_area = spawn_node(
                commands,
                popup,
                Node {
                    position_type: PositionType::Absolute,
                    right: px(5),
                    top: px(4),
                    bottom: px(4),
                    width: px(66),
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    row_gap: px(-2),
                    ..default()
                },
                None,
            );
            commands
                .entity(score_area)
                .insert((ZIndex(3), FocusPolicy::Pass));
            add_text(commands, score_area, "得分", 10.0, MUTED, assets);
            let score = add_text(
                commands,
                score_area,
                displayed_score.to_string(),
                30.0,
                ACCENT,
                assets,
            );
            commands.entity(score).insert((
                PlayerGameScoreText::Own(player.id),
                TextShadow {
                    offset: Vec2::new(1.5, 2.0),
                    color: Color::BLACK.with_alpha(0.82),
                },
            ));
        }
        ScoreCardsPopupPlacement::Opponent(_) => {
            let title = format!(
                "{} 的分牌 · {} 分 · {} 张",
                player.name,
                displayed_score,
                cards.len()
            );
            add_text(commands, popup, title, 12.0, ACCENT, assets);
        }
    }
    if cards.is_empty() {
        if matches!(placement, ScoreCardsPopupPlacement::Opponent(_)) {
            add_text(commands, popup, "尚未获得分牌", 12.0, MUTED, assets);
        }
        return popup;
    }

    let mut displayed_cards = cards.to_vec();
    sort_cards_high_to_low(&mut displayed_cards);
    for chunk in displayed_cards.chunks(24) {
        let row = spawn_node(
            commands,
            popup,
            Node {
                width: percent(100),
                height: px(CardSize::Score.dimensions().1),
                flex_direction: FlexDirection::Row,
                flex_wrap: FlexWrap::NoWrap,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::FlexStart,
                ..default()
            },
            None,
        );
        let last_card = chunk.len().saturating_sub(1);
        for (index, card) in chunk.iter().enumerate() {
            add_card_image(
                commands,
                row,
                *card,
                CardSize::Score,
                index,
                index == last_card,
                false,
                assets,
            );
        }
    }
    popup
}

fn add_round_play_for_optional_player(
    commands: &mut Commands,
    parent: Entity,
    side: SeatSide,
    game: &leocard_protocol::QiGui523Snapshot,
    player: Option<&PlayerPublicState>,
    play_effect: Option<&ActivePlayEffect>,
    last_play: Option<&(PlayerId, PublicPlay)>,
    assets: &UiAssets,
) {
    let justify_content = match side {
        SeatSide::Left => JustifyContent::FlexStart,
        SeatSide::Top => JustifyContent::Center,
        SeatSide::Right => JustifyContent::FlexEnd,
    };
    let play = spawn_round_play_container(commands, parent, justify_content);
    if let Some(player) = player {
        add_round_play(
            commands,
            play,
            game,
            player.id,
            play_effect,
            last_play,
            assets,
        );
    }
}

fn spawn_round_play_container(
    commands: &mut Commands,
    parent: Entity,
    justify_content: JustifyContent,
) -> Entity {
    spawn_node(
        commands,
        parent,
        Node {
            width: px(148),
            min_width: px(148),
            max_width: px(148),
            min_height: px(116),
            flex_shrink: 0.0,
            position_type: PositionType::Relative,
            flex_direction: FlexDirection::Row,
            flex_wrap: FlexWrap::NoWrap,
            align_items: AlignItems::Center,
            justify_content,
            ..default()
        },
        None,
    )
}

fn add_round_play(
    commands: &mut Commands,
    parent: Entity,
    game: &leocard_protocol::QiGui523Snapshot,
    player: leocard_protocol::PlayerId,
    play_effect: Option<&ActivePlayEffect>,
    last_play: Option<&(PlayerId, PublicPlay)>,
    assets: &UiAssets,
) {
    if matches!(&game.phase, GamePhaseView::Finished { .. }) {
        if game
            .players
            .iter()
            .find(|state| state.id == player)
            .is_some_and(|state| state.ready)
        {
            add_text(commands, parent, "准备", 24.0, READY, assets);
            return;
        }
    }
    let is_current_player = matches!(&game.phase, GamePhaseView::Playing)
        && game
            .trick
            .as_ref()
            .is_some_and(|trick| trick.current_player == player);
    if is_current_player {
        if turn_clock_visible(game, player) {
            add_turn_clock(
                commands,
                parent,
                game.turn_timer.filter(|timer| timer.player == player),
                assets,
            );
        }
        return;
    }
    let cards = if matches!(&game.phase, GamePhaseView::Finished { .. }) {
        last_play.and_then(|(record_player, play)| {
            (*record_player == player).then_some(play.cards.as_slice())
        })
    } else {
        game.trick.as_ref().and_then(|trick| {
            trick.records.iter().rev().find_map(|record| match record {
                PublicPlayRecord::Played {
                    player: record_player,
                    play,
                } if *record_player == player => Some(play.cards.as_slice()),
                PublicPlayRecord::Passed {
                    player: record_player,
                } if *record_player == player => Some(&[][..]),
                _ => None,
            })
        })
    };
    let Some(cards) = cards.filter(|cards| !cards.is_empty()) else {
        if !matches!(&game.phase, GamePhaseView::Finished { .. }) {
            add_text(commands, parent, "不出", 22.0, MUTED, assets);
        }
        return;
    };
    let sequence_style = play_effect
        .filter(|effect| effect.player == player && effect.play.cards.len() == cards.len())
        .and_then(|effect| sequence_effect_style(&effect.play.kind));
    let mut displayed_cards = cards.to_vec();
    sort_cards_high_to_low(&mut displayed_cards);
    let last_card = displayed_cards.len().saturating_sub(1);
    let (card_width, card_height) = CardSize::Seat.dimensions();
    let cards_width = card_width + TABLE_CARD_REVEAL * last_card as f32;
    let card_group = spawn_node(
        commands,
        parent,
        Node {
            width: px(cards_width),
            min_width: px(cards_width),
            max_width: px(cards_width),
            height: px(card_height),
            min_height: px(card_height),
            max_height: px(card_height),
            flex_shrink: 0.0,
            position_type: PositionType::Relative,
            flex_direction: FlexDirection::Row,
            flex_wrap: FlexWrap::NoWrap,
            ..default()
        },
        None,
    );
    for (index, card) in displayed_cards.iter().enumerate() {
        let card_entity = add_card_image(
            commands,
            card_group,
            *card,
            CardSize::Seat,
            index,
            index == last_card,
            sequence_style.is_some(),
            assets,
        );
        if sequence_style.is_some() {
            commands.entity(card_entity).insert((
                SequenceEffectCard { index },
                UiTransform::IDENTITY,
                FocusPolicy::Pass,
            ));
        }
    }
    if let Some((label, color, motif)) = sequence_style {
        add_sequence_play_decoration(
            commands,
            card_group,
            displayed_cards.len(),
            label,
            color,
            motif,
            assets,
        );
    }
}

pub(in crate::app) fn turn_clock_visible(
    game: &leocard_protocol::QiGui523Snapshot,
    player: PlayerId,
) -> bool {
    matches!(&game.phase, GamePhaseView::Playing)
        && game
            .trick
            .as_ref()
            .is_some_and(|trick| trick.current_player == player)
        && game
            .players
            .iter()
            .find(|state| state.id == player)
            .is_some_and(|state| !state.auto_play)
}

fn add_turn_clock(
    commands: &mut Commands,
    parent: Entity,
    timer: Option<TurnTimerView>,
    assets: &UiAssets,
) {
    let clock = commands
        .spawn((
            TurnClock,
            Node {
                width: px(40),
                height: px(40),
                margin: UiRect::right(px(12)),
                border: UiRect::all(px(2)),
                border_radius: BorderRadius::all(percent(50)),
                position_type: PositionType::Relative,
                ..default()
            },
            BackgroundColor(HEADER_BG),
            BorderColor::all(ACCENT),
            UiTransform::IDENTITY,
        ))
        .id();
    commands.entity(parent).add_child(clock);

    for (left, rotation) in [(3.0, -0.25), (27.0, 0.25)] {
        let bell = commands
            .spawn((
                Node {
                    position_type: PositionType::Absolute,
                    left: px(left),
                    top: px(-4),
                    width: px(10),
                    height: px(5),
                    border_radius: BorderRadius::all(px(3)),
                    ..default()
                },
                BackgroundColor(ACCENT),
                UiTransform::from_rotation(Rot2::radians(rotation)),
            ))
            .id();
        commands.entity(clock).add_child(bell);
    }

    let hand = commands
        .spawn((
            TurnClockHand,
            Node {
                position_type: PositionType::Absolute,
                left: px(18),
                top: px(8),
                width: px(2),
                height: px(22),
                border_radius: BorderRadius::all(px(1)),
                ..default()
            },
            BackgroundColor(ACCENT),
            UiTransform::IDENTITY,
        ))
        .id();
    commands.entity(clock).add_child(hand);
    let center = spawn_node(
        commands,
        clock,
        Node {
            position_type: PositionType::Absolute,
            left: px(16),
            top: px(16),
            width: px(6),
            height: px(6),
            border_radius: BorderRadius::all(percent(50)),
            ..default()
        },
        Some(TEXT),
    );
    commands.entity(center).insert(ZIndex(2));
    let label = turn_timer_label(timer);
    let label = add_text(commands, parent, label, 22.0, ACCENT, assets);
    commands.entity(label).insert(TurnClockLabel);
}

pub(in crate::app) fn turn_timer_label(timer: Option<TurnTimerView>) -> String {
    match timer {
        Some(timer) if timer.base_seconds > 0 => timer.base_seconds.to_string(),
        Some(timer) => format!("烧条中... {}", timer.reserve_seconds),
        None => String::new(),
    }
}

#[derive(Clone, Copy)]
pub(in crate::app) enum CardSize {
    Hand,
    Seat,
    Score,
    TableScore,
    FinishedHand,
}

pub(in crate::app) fn sort_cards_high_to_low(cards: &mut [Card]) {
    cards.sort_by(|left, right| {
        right
            .rank()
            .strength()
            .cmp(&left.rank().strength())
            .then_with(|| right.suit().strength().cmp(&left.suit().strength()))
            .then_with(|| right.deck().cmp(&left.deck()))
    });
}

impl CardSize {
    pub(in crate::app) fn dimensions(self) -> (f32, f32) {
        match self {
            Self::Hand => (76.0, 103.0),
            Self::Seat => (72.0, 98.0),
            Self::Score => (36.0, 49.0),
            Self::TableScore => (28.0, 38.0),
            Self::FinishedHand => (43.2, 58.8),
        }
    }
}

struct HandCardSpec {
    pub(in crate::app) card: Card,
    pub(in crate::app) selected: bool,
    pub(in crate::app) animation: CardAnimationState,
    pub(in crate::app) index: usize,
    pub(in crate::app) hand_len: usize,
    pub(in crate::app) is_last: bool,
}

fn add_card_button(commands: &mut Commands, parent: Entity, spec: HandCardSpec, assets: &UiAssets) {
    let HandCardSpec {
        card,
        selected,
        animation,
        index,
        hand_len,
        is_last,
    } = spec;
    let (width, height) = CardSize::Hand.dimensions();
    let image = assets
        .cards
        .get(&(card.rank(), card.suit()))
        .expect("all valid card faces are preloaded")
        .clone();
    let initial_pose = hand_card_pose(
        index,
        hand_len,
        animation.face_hover_amount,
        animation.selected_amount,
        animation.deal_elapsed,
        animation.dealing,
    );
    let initial_glow =
        (animation.face_hover_amount * 0.72 + animation.selected_amount * 0.72).clamp(0.0, 1.0);
    let button = commands
        .spawn((
            Button,
            UiAction::ToggleCard,
            HandCardSlot {
                card,
                index,
                is_last,
                hover_amount: animation.slot_hover_amount,
            },
            RelativeCursorPosition::default(),
            Node {
                width: px(if is_last { width } else { HAND_CARD_REVEAL }),
                height: px(height),
                ..default()
            },
        ))
        .id();
    commands.entity(parent).add_child(button);

    let card_face = commands
        .spawn((
            HandCardVisual {
                button,
                card,
                index,
                selected,
                hover_amount: animation.face_hover_amount,
                selected_amount: animation.selected_amount,
                deal_elapsed: animation.deal_elapsed,
                dealing: animation.dealing,
                hand_len,
            },
            Node {
                position_type: PositionType::Absolute,
                left: px(0),
                bottom: px(0),
                width: px(width),
                height: px(height),
                border: UiRect::all(px(2)),
                border_radius: BorderRadius::all(px(6)),
                ..default()
            },
            UiTransform {
                translation: initial_pose.translation,
                rotation: initial_pose.rotation,
                ..UiTransform::IDENTITY
            },
            ImageNode::new(image).with_color(Color::srgb(
                1.0,
                1.0 - initial_glow * 0.035,
                1.0 - initial_glow * 0.16,
            )),
            BorderColor::all(if selected { ACCENT } else { BORDER }),
            Outline::new(
                px(0.75 + initial_glow * 1.5),
                px(0),
                ACCENT.with_alpha(initial_glow * 0.92),
            ),
            BoxShadow::new(Color::NONE, px(0), px(0), px(0), px(3)),
            GlobalZIndex(index as i32 + 1),
            FocusPolicy::Pass,
        ))
        .id();
    commands.entity(button).add_child(card_face);
    let overlay = commands
        .spawn((
            HandCardSelectionOverlay { index },
            Node {
                position_type: PositionType::Absolute,
                left: px(0),
                top: px(0),
                width: percent(100),
                height: percent(100),
                border_radius: BorderRadius::all(px(5)),
                ..default()
            },
            BackgroundColor(Color::NONE),
            FocusPolicy::Pass,
        ))
        .id();
    commands.entity(card_face).add_child(overlay);
}
