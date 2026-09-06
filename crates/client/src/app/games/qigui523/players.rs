use super::*;
use leocard_protocol::QiGui523Snapshot;

pub(super) fn add_opponent_slot(
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
            commands
                .entity(badge)
                .insert((Button, UiAction::ToggleInteractionMenu(player.id)));
            attach_start_game_seat_transition(
                commands,
                badge,
                player.id,
                visuals.start_transition_active,
            );
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
                PlayerMenuProfile {
                    name: &player.name,
                    avatar: handle,
                    reference_points: player.reference_points,
                    completed_games: player.completed_games,
                    game_profiles: &player.game_profiles,
                },
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

fn finished_remaining_hand(game: &QiGui523Snapshot, player: PlayerId) -> Option<&[QiGuiCard]> {
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
    cards: &[QiGuiCard],
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
