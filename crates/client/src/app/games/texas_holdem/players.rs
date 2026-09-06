use super::*;

pub(super) fn add_texas_opponent(
    commands: &mut Commands,
    table: Entity,
    player: &TexasHoldemPlayerState,
    relative: u8,
    game: &TexasHoldemSnapshot,
    interaction_menu_open: Option<PlayerId>,
    assets: &UiAssets,
    avatars: &AvatarImages,
    turn_border_materials: &mut Assets<TurnBorderMaterial>,
    chip_state: &TexasChipTableState,
    start_transition_active: bool,
) {
    let (left, right, top, bottom, side) = match relative {
        // 两侧座位按牌桌高度定位，既向中部收拢，也能随窗口高度稳定缩放。
        1 => (px(18), Val::Auto, Val::Auto, percent(30), SeatSide::Left),
        2 => (px(18), Val::Auto, percent(25), Val::Auto, SeatSide::Left),
        3 => (px(528), Val::Auto, px(26), Val::Auto, SeatSide::Top),
        4 => (Val::Auto, px(18), percent(25), Val::Auto, SeatSide::Right),
        5 => (Val::Auto, px(18), Val::Auto, percent(30), SeatSide::Right),
        _ => return,
    };
    let mut node = Node {
        position_type: PositionType::Absolute,
        width: px(224),
        min_width: px(224),
        height: px(72),
        min_height: px(72),
        max_height: px(72),
        flex_shrink: 0.0,
        padding: match side {
            SeatSide::Left | SeatSide::Top => UiRect::new(px(8), px(68), px(4), px(4)),
            SeatSide::Right => UiRect::new(px(68), px(8), px(4), px(4)),
        },
        flex_direction: FlexDirection::Row,
        align_items: AlignItems::Center,
        column_gap: px(8),
        border: UiRect::all(px(TURN_BORDER_THICKNESS)),
        border_radius: BorderRadius::all(px(8)),
        ..default()
    };
    node.left = left;
    node.right = right;
    node.top = top;
    node.bottom = bottom;
    let base_border = texas_player_border_color(player, game.current_player == Some(player.id));
    let panel = commands
        .spawn((
            Button,
            UiAction::ToggleInteractionMenu(player.id),
            node,
            BackgroundColor(HEADER_BG.with_alpha(if player.folded { 0.62 } else { 0.94 })),
            BorderColor::all(base_border),
            BoxShadow::new(Color::NONE, px(0), px(0), px(0), px(0)),
        ))
        .id();
    commands.entity(table).add_child(panel);
    commands.entity(panel).insert(TexasPlayerPanel {
        player: player.id,
        base_border,
    });
    attach_start_game_seat_transition(commands, panel, player.id, start_transition_active);
    decorate_player_panel(commands, panel, assets, 1.0);
    if !player.folded && game.current_player == Some(player.id) {
        add_turn_border_trace(
            commands,
            panel,
            turn_border_materials,
            TurnBorderAnimationKey::new(GameKind::TexasHoldem, game.match_id, player.id),
        );
    }
    let avatar_handle = player.avatar.and_then(|id| avatars.remote.get(&id));
    let avatar = add_avatar(commands, panel, &player.name, avatar_handle, 32.0, assets);
    commands
        .entity(avatar)
        .insert(PlayerAvatarAnchor(player.id));
    if player.auto_play {
        add_auto_play_robot_indicator(commands, panel, player.id, side, assets);
    }
    add_role_tokens(commands, avatar, player.id, game, assets);
    let info = spawn_node(
        commands,
        panel,
        Node {
            flex_direction: FlexDirection::Column,
            row_gap: px(1),
            width: px(82),
            min_width: px(82),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            ..default()
        },
        None,
    );
    add_text(commands, info, &player.name, 14.0, TEXT, assets);
    add_text(
        commands,
        info,
        texas_player_status(player),
        12.0,
        MUTED,
        assets,
    );
    add_player_panel_primary_value(commands, panel, side, player.stack, assets);

    let popup = add_texas_chip_popup(
        commands,
        panel,
        &player.name,
        player.stack,
        Some(side),
        assets,
        &chip_state.stack_counts(player.id),
    );
    commands.entity(popup).insert(Visibility::Hidden);
    let menu = add_interaction_menu(
        commands,
        panel,
        player.id,
        side,
        PlayerMenuProfile {
            name: &player.name,
            avatar: avatar_handle,
            reference_points: player.reference_points,
            completed_games: player.completed_games,
            game_profiles: &player.game_profiles,
        },
        assets,
    );
    commands
        .entity(menu)
        .insert(if interaction_menu_open == Some(player.id) {
            Visibility::Visible
        } else {
            Visibility::Hidden
        });
    commands.entity(panel).insert(OpponentBadge {
        player: player.id,
        score_popup: Some(popup),
        interaction_menu: menu,
    });
}

pub(super) fn add_role_tokens(
    commands: &mut Commands,
    avatar: Entity,
    player: PlayerId,
    game: &TexasHoldemSnapshot,
    assets: &UiAssets,
) {
    let mut labels = Vec::new();
    if player == game.dealer {
        labels.push("D");
    }
    if player == game.small_blind {
        labels.push("SB");
    }
    if player == game.big_blind {
        labels.push("BB");
    }
    if labels.is_empty() {
        return;
    }
    let row = spawn_node(
        commands,
        avatar,
        Node {
            position_type: PositionType::Absolute,
            right: px(-9),
            bottom: px(-7),
            flex_direction: FlexDirection::Row,
            column_gap: px(2),
            ..default()
        },
        None,
    );
    commands.entity(row).insert((ZIndex(45), FocusPolicy::Pass));
    for label in labels {
        let chip = spawn_node(
            commands,
            row,
            Node {
                min_width: px(18),
                height: px(18),
                padding: UiRect::horizontal(px(3)),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                border_radius: BorderRadius::all(percent(50)),
                ..default()
            },
            Some(ACCENT),
        );
        commands.entity(chip).insert(FocusPolicy::Pass);
        add_text(commands, chip, label, 8.0, HEADER_BG, assets);
    }
}

/// 德州筹码面板沿用七鬼五二三分牌面板的外观，并显示账本中真实存在的筹码。
pub(super) fn add_texas_chip_popup(
    commands: &mut Commands,
    parent: Entity,
    player_name: &str,
    stack: u32,
    opponent_side: Option<SeatSide>,
    assets: &UiAssets,
    stack_counts: &[(u16, usize)],
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
    if let Some(side) = opponent_side {
        position_opponent_popup(&mut node, side);
    } else {
        node.left = px(10);
        node.bottom = px(64);
        node.width = px(410);
        node.min_height = px(58);
        node.padding = UiRect::new(px(6), px(76), px(6), px(6));
    }
    let popup = spawn_node(commands, parent, node, Some(Color::BLACK.with_alpha(0.30)));
    commands.entity(popup).insert((
        BorderColor::all(ACCENT.with_alpha(0.72)),
        GlobalZIndex(1500),
        FocusPolicy::Pass,
    ));

    if opponent_side.is_some() {
        add_text(
            commands,
            popup,
            format!("{player_name} 的筹码 · 剩余 {stack}"),
            12.0,
            ACCENT,
            assets,
        );
    } else {
        let value_area = spawn_node(
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
            .entity(value_area)
            .insert((ZIndex(3), FocusPolicy::Pass));
        add_text(commands, value_area, "筹码", 10.0, MUTED, assets);
        let value = add_text(
            commands,
            value_area,
            stack.to_string(),
            30.0,
            ACCENT,
            assets,
        );
        commands.entity(value).insert(TextShadow {
            offset: Vec2::new(1.5, 2.0),
            color: Color::BLACK.with_alpha(0.82),
        });
    }

    let chips = spawn_node(
        commands,
        popup,
        Node {
            width: percent(100),
            height: px(38),
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::Center,
            justify_content: JustifyContent::FlexStart,
            column_gap: px(18),
            ..default()
        },
        None,
    );
    for &(denomination, count) in stack_counts {
        add_horizontal_chip_group(commands, chips, denomination, count, assets);
    }
    popup
}

/// 同面值筹码水平紧叠，不同面值由父节点的 column_gap 分组隔开。
fn add_horizontal_chip_group(
    commands: &mut Commands,
    parent: Entity,
    denomination: u16,
    count: usize,
    assets: &UiAssets,
) {
    const CHIP_SIZE: f32 = 34.0;
    const CHIP_REVEAL: f32 = 11.0;
    let group_width = CHIP_SIZE + count.saturating_sub(1) as f32 * CHIP_REVEAL;
    let group = spawn_node(
        commands,
        parent,
        Node {
            position_type: PositionType::Relative,
            width: px(group_width),
            height: px(CHIP_SIZE),
            flex_shrink: 0.0,
            ..default()
        },
        None,
    );
    let Some(image) = assets.games.poker_chips.get(&denomination) else {
        return;
    };
    for index in 0..count {
        let chip = commands
            .spawn((
                Node {
                    position_type: PositionType::Absolute,
                    left: px(index as f32 * CHIP_REVEAL),
                    top: px(0),
                    width: px(CHIP_SIZE),
                    height: px(CHIP_SIZE),
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    ..default()
                },
                ImageNode::new(image.clone()),
                ZIndex(index as i32),
                FocusPolicy::Pass,
            ))
            .id();
        commands.entity(group).add_child(chip);
        let label_color = if denomination == 1 { HEADER_BG } else { TEXT };
        let label = add_text(
            commands,
            chip,
            denomination.to_string(),
            9.0,
            label_color,
            assets,
        );
        commands.entity(label).insert(TextShadow {
            offset: Vec2::new(0.7, 0.8),
            color: Color::BLACK.with_alpha(0.55),
        });
    }
}
