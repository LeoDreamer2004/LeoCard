//! 德州扑克牌桌视图。房间、聊天、头像、桌布和按钮资源均复用公共客户端层。

use super::*;

pub(in crate::app) struct TexasTableVisuals<'a> {
    pub(in crate::app) assets: &'a UiAssets,
    pub(in crate::app) avatars: &'a AvatarImages,
    pub(in crate::app) appearance: &'a TableAppearance,
    pub(in crate::app) brightness: f32,
    pub(in crate::app) vignette: f32,
    pub(in crate::app) table_materials: &'a mut Assets<TableBackgroundMaterial>,
    pub(in crate::app) turn_border_materials: &'a mut Assets<TurnBorderMaterial>,
    pub(in crate::app) chip_zone_materials: &'a mut Assets<TexasChipZoneMaterial>,
    pub(in crate::app) chip_state: &'a TexasChipTableState,
    pub(in crate::app) game_summary: &'a GameSummaryAnimation,
}

#[derive(Component)]
pub(in crate::app) struct TexasDealCard {
    elapsed: f32,
    delay: f32,
    offset: Vec2,
}

#[derive(Component)]
pub(in crate::app) struct TexasFlyingCardBack {
    elapsed: f32,
    delay: f32,
    source: Vec2,
    target: Vec2,
}

#[derive(Component)]
pub(in crate::app) struct TexasBoardCardBack {
    elapsed: f32,
    delay: f32,
    start_offset: Vec2,
    phase: f32,
}

#[derive(Component)]
pub(in crate::app) struct TexasBoardCardFlip {
    elapsed: f32,
    delay: f32,
    face: Handle<Image>,
    face_visible: bool,
}

struct TexasInitialDeal {
    duration: f32,
    own_delays: [f32; 2],
}

pub(in crate::app) fn render_texas_holdem_table(
    commands: &mut Commands,
    root: Entity,
    client: &ClientResource,
    game: &TexasHoldemSnapshot,
    ui: &mut UiState,
    chat: &ChatPanelState,
    visuals: TexasTableVisuals<'_>,
) {
    let TexasTableVisuals {
        assets,
        avatars,
        appearance,
        brightness,
        vignette,
        table_materials,
        turn_border_materials,
        chip_zone_materials,
        chip_state,
        game_summary,
    } = visuals;
    let content = spawn_node(
        commands,
        root,
        Node {
            width: percent(100),
            flex_grow: 1.0,
            min_height: px(0),
            position_type: PositionType::Relative,
            ..default()
        },
        None,
    );
    let felt = appearance
        .custom_felt
        .as_ref()
        .unwrap_or(&assets.table_felt)
        .clone();
    let background_params =
        table_material_params(brightness, vignette, appearance.custom_felt.is_none());
    let material = table_materials.add(TableBackgroundMaterial {
        params: background_params,
        texture: felt.clone(),
    });
    commands
        .entity(content)
        .insert((MaterialNode(material.clone()), TableBackground));

    let table = spawn_node(
        commands,
        content,
        Node {
            position_type: PositionType::Absolute,
            left: percent(50),
            width: px(DESIGN_WIDTH),
            top: px(0),
            bottom: px(0),
            min_height: px(430),
            ..default()
        },
        None,
    );
    commands.entity(table).insert((
        UiTransform::from_translation(Val2::px(-DESIGN_WIDTH * 0.5, 0.0)),
        // 中央设计画布重新铺同一背景，使筹码区可按牌桌坐标准确取样。
        MaterialNode(material),
    ));
    if let NetworkState::Reconnecting(message) = client.0.state() {
        add_reconnecting_overlay(commands, table, message, assets);
    }

    let own = game
        .players
        .iter()
        .find(|player| player.id == game.you)
        .expect("德州快照必须包含接收方");
    for relative in 1..TABLE_SEAT_COUNT {
        let physical = SeatId((own.seat.0 + relative) % TABLE_SEAT_COUNT);
        if let Some(player) = game.players.iter().find(|player| player.seat == physical) {
            add_texas_opponent(
                commands,
                table,
                player,
                relative,
                game,
                ui.interaction_menu_open,
                assets,
                avatars,
                turn_border_materials,
                chip_state,
            );
        }
    }

    let new_hand = ui.texas_observed_match != Some(game.match_id)
        || ui.texas_observed_hand_number != game.hand_number;
    let new_community_from = if new_hand {
        0
    } else {
        ui.texas_observed_community_len.min(game.community.len())
    };
    let initial_deal =
        new_hand.then(|| spawn_texas_initial_deal(commands, table, game, own.seat, assets));
    add_texas_chip_areas(
        commands,
        table,
        game,
        chip_state,
        assets,
        chip_zone_materials,
        &felt,
        appearance.custom_felt.is_none(),
        background_params.y,
    );
    add_community_area(
        commands,
        table,
        game,
        new_community_from,
        initial_deal.as_ref().map(|deal| deal.duration),
        assets,
    );
    add_texas_own_area(
        commands,
        table,
        game,
        own,
        initial_deal.as_ref().map(|deal| deal.own_delays),
        ui,
        assets,
        avatars,
        turn_border_materials,
        chip_state,
    );
    add_texas_showdown_reveal(commands, table, game, own.seat, assets, game_summary);
    add_texas_hand_result(commands, table, game, assets, avatars, game_summary);

    let local_auto_play =
        matches!(game.phase, TexasHoldemPhaseView::Betting { .. }).then_some(own.auto_play);
    add_chat_panel(commands, content, chat, assets, local_auto_play, None, None);
    if local_auto_play == Some(true) {
        add_auto_play_overlay(commands, content, assets);
    }

    ui.texas_observed_match = Some(game.match_id);
    ui.texas_observed_hand_number = game.hand_number;
    ui.texas_observed_community_len = game.community.len();
}

fn add_community_area(
    commands: &mut Commands,
    table: Entity,
    game: &TexasHoldemSnapshot,
    new_from: usize,
    initial_deal_duration: Option<f32>,
    assets: &UiAssets,
) {
    let center = spawn_node(
        commands,
        table,
        Node {
            position_type: PositionType::Absolute,
            left: percent(50),
            top: px(206),
            width: px(560),
            height: px(90),
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            ..default()
        },
        None,
    );
    commands.entity(center).insert((
        UiTransform::from_translation(Val2::px(-280.0, 0.0)),
        // 公共牌始终绘制在筹码区背景框之上。
        ZIndex(10),
    ));
    let cards = spawn_node(
        commands,
        center,
        Node {
            width: percent(100),
            height: px(90),
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            column_gap: px(7),
            ..default()
        },
        None,
    );
    let hidden_community = 5_usize.saturating_sub(game.community.len());
    let visual_draw_pile = usize::from(game.draw_pile_len).saturating_sub(hidden_community);
    let pile_column = spawn_node(
        commands,
        cards,
        Node {
            width: px(46),
            height: px(90),
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            row_gap: px(2),
            flex_shrink: 0.0,
            ..default()
        },
        None,
    );
    add_texas_draw_pile(commands, pile_column, visual_draw_pile, assets);
    add_text(
        commands,
        pile_column,
        street_label(&game.phase),
        12.0,
        TEXT,
        assets,
    );
    let gap = spawn_node(
        commands,
        cards,
        Node {
            width: px(8),
            height: px(1),
            ..default()
        },
        None,
    );
    commands.entity(gap).insert(FocusPolicy::Pass);
    for index in 0..5 {
        if let Some(card) = game.community.get(index).copied() {
            let animate = index >= new_from;
            add_texas_board_face(
                commands,
                cards,
                card,
                animate.then_some(index.saturating_sub(new_from) as f32 * 0.08),
                assets,
            );
        } else {
            add_texas_board_back(
                commands,
                cards,
                index,
                initial_deal_duration.map(|duration| duration + index as f32 * 0.065),
                assets,
            );
        }
    }
}

fn add_texas_draw_pile(commands: &mut Commands, parent: Entity, count: usize, assets: &UiAssets) {
    let pile = spawn_node(
        commands,
        parent,
        Node {
            width: px(44),
            height: px(62),
            position_type: PositionType::Relative,
            ..default()
        },
        None,
    );
    for layer in 0..5 {
        let card = commands
            .spawn((
                Node {
                    position_type: PositionType::Absolute,
                    left: px(layer as f32 * 1.7),
                    top: px((4 - layer) as f32 * 1.0),
                    width: px(39),
                    height: px(54),
                    border: UiRect::all(px(1)),
                    border_radius: BorderRadius::all(px(4)),
                    ..default()
                },
                ImageNode::new(assets.card_back.clone()),
                BorderColor::all(TEXT.with_alpha(0.52)),
                ZIndex(layer),
            ))
            .id();
        commands.entity(pile).add_child(card);
    }
    let counter = spawn_node(
        commands,
        pile,
        Node {
            position_type: PositionType::Absolute,
            left: px(7),
            top: px(27),
            width: px(29),
            height: px(21),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            border_radius: BorderRadius::all(px(6)),
            ..default()
        },
        Some(Color::BLACK.with_alpha(0.68)),
    );
    commands.entity(counter).insert(ZIndex(8));
    add_text(commands, counter, count.to_string(), 12.0, TEXT, assets);
}

fn add_texas_board_back(
    commands: &mut Commands,
    parent: Entity,
    index: usize,
    delay: Option<f32>,
    assets: &UiAssets,
) {
    let animated = delay.is_some();
    let card = commands
        .spawn((
            Node {
                width: px(56),
                height: px(76),
                border: UiRect::all(px(1)),
                border_radius: BorderRadius::all(px(5)),
                ..default()
            },
            ImageNode::new(assets.card_back.clone()).with_color(if animated {
                Color::WHITE.with_alpha(0.0)
            } else {
                Color::WHITE
            }),
            UiTransform::IDENTITY,
        ))
        .id();
    commands.entity(parent).add_child(card);
    commands.entity(card).insert(TexasBoardCardBack {
        elapsed: if animated { 0.0 } else { 1.0 },
        delay: delay.unwrap_or(0.0),
        start_offset: if animated {
            Vec2::new(-65.0 - index as f32 * 63.0, 0.0)
        } else {
            Vec2::ZERO
        },
        phase: index as f32 * 0.9,
    });
    if let Some(delay) = delay {
        commands.spawn(PendingDealSound {
            remaining: delay,
            variant: index % assets.deal_sounds.len().max(1),
        });
    }
}

fn add_texas_board_face(
    commands: &mut Commands,
    parent: Entity,
    card: TexasHoldemCard,
    flip_delay: Option<f32>,
    assets: &UiAssets,
) {
    let face = texas_card_face(card, assets);
    let entity = commands
        .spawn((
            Node {
                width: px(56),
                height: px(76),
                border: UiRect::all(px(1)),
                border_radius: BorderRadius::all(px(5)),
                ..default()
            },
            ImageNode::new(if flip_delay.is_some() {
                assets.card_back.clone()
            } else {
                face.clone()
            }),
            BorderColor::all(TEXT.with_alpha(0.42)),
            UiTransform::IDENTITY,
        ))
        .id();
    commands.entity(parent).add_child(entity);
    if let Some(delay) = flip_delay {
        commands.entity(entity).insert(TexasBoardCardFlip {
            elapsed: 0.0,
            delay,
            face,
            face_visible: false,
        });
    }
}

fn spawn_texas_initial_deal(
    commands: &mut Commands,
    table: Entity,
    game: &TexasHoldemSnapshot,
    own_seat: SeatId,
    assets: &UiAssets,
) -> TexasInitialDeal {
    let source = Vec2::new(456.0, 258.0);
    let dealer_seat = game
        .players
        .iter()
        .find(|player| player.id == game.dealer)
        .map_or(0, |player| player.seat.0);
    let mut funded = game
        .players
        .iter()
        .filter(|player| !player.folded)
        .collect::<Vec<_>>();
    funded
        .sort_by_key(|player| (player.seat.0 + TABLE_SEAT_COUNT - dealer_seat) % TABLE_SEAT_COUNT);
    if !funded.is_empty() {
        funded.rotate_left(1);
    }
    let mut dealt = 0usize;
    let mut own_delays = [0.0; 2];
    let mut own_card = 0usize;
    for _ in 0..2 {
        for player in &funded {
            if player.id == game.you {
                if own_card < own_delays.len() {
                    own_delays[own_card] = dealt as f32 * 0.07;
                    own_card += 1;
                }
                dealt += 1;
                continue;
            }
            let relative = (player.seat.0 + TABLE_SEAT_COUNT - own_seat.0) % TABLE_SEAT_COUNT;
            let target = texas_seat_card_target(relative);
            let delay = dealt as f32 * 0.07;
            let entity = commands
                .spawn((
                    Node {
                        position_type: PositionType::Absolute,
                        left: px(source.x),
                        top: px(source.y),
                        width: px(36),
                        height: px(49),
                        border_radius: BorderRadius::all(px(3)),
                        ..default()
                    },
                    ImageNode::new(assets.card_back.clone())
                        .with_color(Color::WHITE.with_alpha(0.0)),
                    UiTransform::IDENTITY,
                    ZIndex(900 + dealt as i32),
                    TexasFlyingCardBack {
                        elapsed: 0.0,
                        delay,
                        source,
                        target,
                    },
                ))
                .id();
            commands.entity(table).add_child(entity);
            commands.spawn(PendingDealSound {
                remaining: delay,
                variant: dealt % assets.deal_sounds.len().max(1),
            });
            dealt += 1;
        }
    }
    TexasInitialDeal {
        duration: dealt as f32 * 0.07 + 0.30,
        own_delays,
    }
}

fn texas_seat_card_target(relative: u8) -> Vec2 {
    match relative {
        0 => Vec2::new(620.0, 570.0),
        1 => Vec2::new(180.0, 400.0),
        2 => Vec2::new(180.0, 175.0),
        3 => Vec2::new(620.0, 50.0),
        4 => Vec2::new(1060.0, 175.0),
        5 => Vec2::new(1060.0, 400.0),
        _ => Vec2::new(620.0, 300.0),
    }
}

fn add_texas_opponent(
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
        &player.name,
        avatar_handle,
        player.reference_points,
        player.completed_games,
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

fn add_role_tokens(
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
fn add_texas_chip_popup(
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
    let Some(image) = assets.poker_chips.get(&denomination) else {
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

fn add_texas_own_area(
    commands: &mut Commands,
    table: Entity,
    game: &TexasHoldemSnapshot,
    own: &TexasHoldemPlayerState,
    deal_delays: Option<[f32; 2]>,
    ui: &mut UiState,
    assets: &UiAssets,
    avatars: &AvatarImages,
    turn_border_materials: &mut Assets<TurnBorderMaterial>,
    chip_state: &TexasChipTableState,
) {
    let own_panel = spawn_node(
        commands,
        table,
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
            flex_shrink: 0.0,
            padding: UiRect::axes(px(3), px(2)),
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            column_gap: px(2),
            border: UiRect::all(px(TURN_BORDER_THICKNESS)),
            border_radius: BorderRadius::all(px(8)),
            ..default()
        },
        Some(HEADER_BG.with_alpha(0.94)),
    );
    let base_border = texas_player_border_color(own, game.current_player == Some(own.id));
    commands.entity(own_panel).insert((
        BorderColor::all(base_border),
        BoxShadow::new(Color::NONE, px(0), px(0), px(0), px(0)),
        TexasPlayerPanel {
            player: own.id,
            base_border,
        },
    ));
    decorate_player_panel(commands, own_panel, assets, 0.72);
    if !own.folded && game.current_player == Some(own.id) {
        add_turn_border_trace(
            commands,
            own_panel,
            turn_border_materials,
            TurnBorderAnimationKey::new(GameKind::TexasHoldem, game.match_id, own.id),
        );
    }
    let avatar_handle = own.avatar.and_then(|id| avatars.remote.get(&id));
    let avatar = add_avatar(commands, own_panel, &own.name, avatar_handle, 24.0, assets);
    commands.entity(avatar).insert(PlayerAvatarAnchor(own.id));
    add_role_tokens(commands, avatar, own.id, game, assets);
    let info = spawn_node(
        commands,
        own_panel,
        Node {
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            ..default()
        },
        None,
    );
    add_text(commands, info, &own.name, 11.0, TEXT, assets);
    add_text(commands, info, texas_player_status(own), 7.5, MUTED, assets);
    add_texas_chip_popup(
        commands,
        table,
        &own.name,
        own.stack,
        None,
        assets,
        &chip_state.stack_counts(own.id),
    );

    // 一手结束后，自己的底牌也和其他玩家一样改放到面前的筹码区。
    if !matches!(game.phase, TexasHoldemPhaseView::HandComplete { .. }) {
        let hole_cards = spawn_node(
            commands,
            table,
            Node {
                position_type: PositionType::Absolute,
                left: percent(50),
                bottom: px(8),
                width: px(180),
                height: px(114),
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::FlexEnd,
                justify_content: JustifyContent::Center,
                column_gap: px(9),
                ..default()
            },
            None,
        );
        commands
            .entity(hole_cards)
            .insert(UiTransform::from_translation(Val2::px(-90.0, 0.0)));
        if !own.folded {
            for (index, card) in game.your_hole_cards.iter().copied().enumerate() {
                let delay = deal_delays.map(|delays| delays[index.min(delays.len() - 1)]);
                let animation = delay.map(|delay| (delay, Vec2::new(-280.0, -245.0)));
                add_texas_card(commands, hole_cards, card, (84.0, 114.0), animation, assets);
                if let Some(delay) = delay {
                    commands.spawn(PendingDealSound {
                        remaining: delay,
                        variant: index % assets.deal_sounds.len().max(1),
                    });
                }
            }
        } else {
            let folded_label = spawn_node(
                commands,
                hole_cards,
                Node {
                    position_type: PositionType::Absolute,
                    left: px(0),
                    right: px(0),
                    top: px(0),
                    bottom: px(0),
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    ..default()
                },
                None,
            );
            commands
                .entity(folded_label)
                .insert((ZIndex(20), FocusPolicy::Pass));
            let label = add_text(commands, folded_label, "您已弃牌", 18.0, MUTED, assets);
            commands.entity(label).insert(TextShadow {
                offset: Vec2::new(1.0, 1.5),
                color: Color::BLACK.with_alpha(0.82),
            });
        }
    }
    add_texas_actions(commands, table, game, own, ui, assets);
}

fn add_texas_actions(
    commands: &mut Commands,
    table: Entity,
    game: &TexasHoldemSnapshot,
    own: &TexasHoldemPlayerState,
    ui: &mut UiState,
    assets: &UiAssets,
) {
    let actions = spawn_node(
        commands,
        table,
        Node {
            position_type: PositionType::Absolute,
            left: percent(50),
            bottom: px(124),
            width: px(620),
            min_height: px(42),
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            row_gap: px(7),
            ..default()
        },
        None,
    );
    commands
        .entity(actions)
        .insert(UiTransform::from_translation(Val2::px(-310.0, 0.0)));
    if !matches!(game.phase, TexasHoldemPhaseView::Betting { .. }) {
        return;
    }
    if let Some(blind) = game.blind_to_post {
        if blind.player == game.you {
            let label = match blind.kind {
                TexasBlindKind::Small => format!("下小盲 {}", blind.amount),
                TexasBlindKind::Big => format!("下大盲 {}", blind.amount),
            };
            let row = spawn_node(
                commands,
                actions,
                Node {
                    width: percent(100),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
                },
                None,
            );
            add_texas_action_button(
                commands,
                row,
                &label,
                TexasHoldemAction::PostBlind,
                ButtonKind::Primary,
                assets,
            );
        } else {
            let player = game
                .players
                .iter()
                .find(|player| player.id == blind.player)
                .map_or("玩家", |player| player.name.as_str());
            add_text(
                commands,
                actions,
                format!("等待 {player} 下盲注…"),
                14.0,
                MUTED,
                assets,
            );
        }
        return;
    }
    if game.current_player != Some(game.you) {
        add_text(commands, actions, "等待其他玩家行动…", 14.0, MUTED, assets);
        return;
    }
    let maximum_target = own.committed_street.saturating_add(own.stack);
    let minimum_target = game.minimum_raise_to.min(maximum_target);
    if ui.texas_raise_to < minimum_target || ui.texas_raise_to > maximum_target {
        ui.texas_raise_to = minimum_target;
    }
    let row = spawn_node(
        commands,
        actions,
        Node {
            width: percent(100),
            flex_direction: FlexDirection::Row,
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            column_gap: px(8),
            ..default()
        },
        None,
    );
    add_texas_action_button(
        commands,
        row,
        "弃牌",
        TexasHoldemAction::Fold,
        ButtonKind::Pass,
        assets,
    );
    if game.amount_to_call == 0 {
        add_texas_action_button(
            commands,
            row,
            "过牌",
            TexasHoldemAction::Check,
            ButtonKind::Secondary,
            assets,
        );
    } else {
        add_texas_action_button(
            commands,
            row,
            &format!("跟注 {}", game.amount_to_call.min(own.stack)),
            TexasHoldemAction::Call,
            ButtonKind::Secondary,
            assets,
        );
    }
    if game.raise_allowed && maximum_target >= game.minimum_raise_to {
        let step = game
            .minimum_raise_to
            .saturating_sub(game.current_bet)
            .max(1);
        let lower = ui.texas_raise_to.saturating_sub(step).max(minimum_target);
        let higher = ui.texas_raise_to.saturating_add(step).min(maximum_target);
        add_raise_adjust_button(
            commands,
            row,
            "−",
            lower,
            lower < ui.texas_raise_to,
            TexasRaiseAdjustButton {
                direction: -1,
                step,
                minimum: minimum_target,
                maximum: maximum_target,
            },
            assets,
        );
        add_texas_action_button(
            commands,
            row,
            &format!("加注到 {}", ui.texas_raise_to),
            TexasHoldemAction::RaiseTo(ui.texas_raise_to),
            ButtonKind::Primary,
            assets,
        );
        add_raise_adjust_button(
            commands,
            row,
            "+",
            higher,
            higher > ui.texas_raise_to,
            TexasRaiseAdjustButton {
                direction: 1,
                step,
                minimum: minimum_target,
                maximum: maximum_target,
            },
            assets,
        );
    }
    add_texas_action_button(
        commands,
        row,
        "全下",
        TexasHoldemAction::AllIn,
        ButtonKind::Warning,
        assets,
    );
}

fn add_texas_action_button(
    commands: &mut Commands,
    parent: Entity,
    label: &str,
    action: TexasHoldemAction,
    kind: ButtonKind,
    assets: &UiAssets,
) {
    add_texas_sized_button(
        commands,
        parent,
        label,
        UiAction::TexasAct(action),
        kind,
        105.0,
        assets,
    );
}

fn add_raise_adjust_button(
    commands: &mut Commands,
    parent: Entity,
    label: &str,
    target: u32,
    enabled: bool,
    repeat: TexasRaiseAdjustButton,
    assets: &UiAssets,
) {
    if enabled {
        let button = add_texas_sized_button(
            commands,
            parent,
            label,
            UiAction::SetTexasRaiseTo(target),
            ButtonKind::Primary,
            34.0,
            assets,
        );
        commands.entity(button).insert(repeat);
    } else {
        add_disabled_texas_sized_button(commands, parent, label, 34.0, assets);
    }
}

fn add_disabled_texas_sized_button(
    commands: &mut Commands,
    parent: Entity,
    label: &str,
    width: f32,
    assets: &UiAssets,
) {
    let button = commands
        .spawn((
            Node {
                width: px(width),
                min_width: px(width),
                height: px(42),
                min_height: px(42),
                padding: UiRect::axes(px(7), px(4)),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                border_radius: BorderRadius::all(px(6)),
                ..default()
            },
            ImageNode::new(assets.secondary_button.clone())
                .with_mode(NodeImageMode::Stretch)
                .with_color(Color::srgb(0.25, 0.29, 0.28)),
            FocusPolicy::Pass,
        ))
        .id();
    commands.entity(parent).add_child(button);
    add_text(
        commands,
        button,
        label,
        15.0,
        MUTED.with_alpha(0.66),
        assets,
    );
}

fn add_texas_sized_button(
    commands: &mut Commands,
    parent: Entity,
    label: &str,
    action: UiAction,
    kind: ButtonKind,
    width: f32,
    assets: &UiAssets,
) -> Entity {
    let (image, normal, hovered, pressed) = match kind {
        ButtonKind::Primary => (
            assets.primary_button.clone(),
            Color::WHITE,
            Color::srgb(1.0, 1.0, 0.82),
            Color::srgb(0.78, 0.90, 0.78),
        ),
        ButtonKind::Secondary => (
            assets.secondary_button.clone(),
            Color::srgb(0.48, 0.62, 0.76),
            Color::srgb(0.64, 0.76, 0.88),
            Color::srgb(0.32, 0.46, 0.60),
        ),
        ButtonKind::Warning => (
            assets.warning_button.clone(),
            Color::WHITE,
            Color::srgb(1.0, 0.94, 0.74),
            Color::srgb(0.90, 0.78, 0.58),
        ),
        ButtonKind::Pass => (
            assets.danger_button.clone(),
            Color::srgb(0.58, 0.42, 0.42),
            Color::srgb(0.76, 0.58, 0.56),
            Color::srgb(0.42, 0.28, 0.27),
        ),
    };
    let button = commands
        .spawn((
            Button,
            action,
            ButtonTint {
                normal,
                hovered,
                pressed,
            },
            Node {
                width: px(width),
                min_width: px(width),
                height: px(42),
                min_height: px(42),
                padding: UiRect::axes(px(7), px(4)),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                border_radius: BorderRadius::all(px(6)),
                ..default()
            },
            ImageNode::new(image)
                .with_mode(NodeImageMode::Stretch)
                .with_color(normal),
        ))
        .id();
    commands.entity(parent).add_child(button);
    let text = add_text(commands, button, label, 15.0, Color::WHITE, assets);
    commands.entity(text).insert((
        Node {
            margin: UiRect::ZERO,
            align_self: AlignSelf::Center,
            ..default()
        },
        FocusPolicy::Pass,
    ));
    button
}

fn add_texas_showdown_reveal(
    commands: &mut Commands,
    table: Entity,
    game: &TexasHoldemSnapshot,
    own_seat: SeatId,
    assets: &UiAssets,
    animation: &GameSummaryAnimation,
) {
    let TexasHoldemPhaseView::HandComplete {
        showdown: true,
        awards,
        ..
    } = &game.phase
    else {
        return;
    };
    if animation.elapsed >= 0.0 {
        return;
    }
    let Some(main_award) = awards.first() else {
        return;
    };
    let Some(winner_id) = main_award.winners.first().copied() else {
        return;
    };
    let Some(winner) = game.players.iter().find(|player| player.id == winner_id) else {
        return;
    };
    let Some(revealed) = game
        .revealed_hands
        .iter()
        .find(|hand| hand.player == winner_id)
    else {
        return;
    };
    let Some(best) = revealed.best else {
        return;
    };

    let root = spawn_node(
        commands,
        table,
        Node {
            position_type: PositionType::Absolute,
            left: px(0),
            right: px(0),
            top: px(0),
            bottom: px(0),
            ..default()
        },
        None,
    );
    commands.entity(root).insert((
        TexasShowdownRevealRoot,
        GlobalZIndex(1080),
        FocusPolicy::Pass,
    ));
    let backdrop = spawn_node(
        commands,
        root,
        Node {
            position_type: PositionType::Absolute,
            left: px(0),
            right: px(0),
            top: px(0),
            bottom: px(0),
            ..default()
        },
        Some(Color::BLACK.with_alpha(0.0)),
    );
    commands
        .entity(backdrop)
        .insert((TexasShowdownBackdrop, FocusPolicy::Pass));

    let relative = (winner.seat.0 + TABLE_SEAT_COUNT - own_seat.0) % TABLE_SEAT_COUNT;
    let winner_zone = texas_player_chip_zone(relative);
    for (index, card) in best.cards().into_iter().enumerate() {
        let hole_index = revealed.cards.iter().position(|hole| *hole == card);
        let (source, delay, start_scale) = if let Some(hole_index) = hole_index {
            (
                Vec2::new(
                    winner_zone.left + winner_zone.width * 0.5 - 31.0 + hole_index as f32 * 31.0,
                    winner_zone.top + 34.0,
                ),
                0.12 + hole_index as f32 * 0.10,
                0.55,
            )
        } else {
            let board_index = game
                .community
                .iter()
                .position(|community| *community == card)
                .unwrap_or(index);
            (
                Vec2::new(520.0 + board_index as f32 * 63.0, 213.0),
                0.68 + board_index as f32 * 0.055,
                0.80,
            )
        };
        let target = Vec2::new(444.0 + index as f32 * 78.0, 238.0);
        let entity = commands
            .spawn((
                Node {
                    position_type: PositionType::Absolute,
                    left: px(target.x),
                    top: px(target.y),
                    width: px(70),
                    height: px(96),
                    border: UiRect::all(px(1)),
                    border_radius: BorderRadius::all(px(6)),
                    ..default()
                },
                ImageNode::new(texas_card_face(card, assets))
                    .with_color(Color::WHITE.with_alpha(0.0)),
                BorderColor::all(Color::srgb(0.88, 0.75, 0.35).with_alpha(0.0)),
                UiTransform {
                    translation: Val2::px(source.x - target.x, source.y - target.y),
                    scale: Vec2::splat(start_scale),
                    ..UiTransform::IDENTITY
                },
                TexasShowdownBestCard {
                    source,
                    target,
                    delay,
                    start_scale,
                },
                ZIndex(index as i32 + 5),
                FocusPolicy::Pass,
            ))
            .id();
        commands.entity(root).add_child(entity);
    }

    let title_area = spawn_node(
        commands,
        root,
        Node {
            position_type: PositionType::Absolute,
            left: px(360),
            top: px(350),
            width: px(560),
            height: px(42),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            ..default()
        },
        None,
    );
    commands.entity(title_area).insert((
        TexasShowdownTitle,
        UiTransform::from_translation(Val2::px(0.0, 16.0)),
        FocusPolicy::Pass,
    ));
    let winner_names = main_award
        .winners
        .iter()
        .filter_map(|id| game.players.iter().find(|player| player.id == *id))
        .map(|player| player.name.as_str())
        .collect::<Vec<_>>()
        .join("、");
    let title = add_text(
        commands,
        title_area,
        format!("{winner_names} · {}", texas_category_label(best.category())),
        30.0,
        Color::NONE,
        assets,
    );
    commands.entity(title).insert(TextShadow {
        offset: Vec2::new(1.2, 1.8),
        color: Color::BLACK.with_alpha(0.72),
    });
    commands.entity(title).insert(TexasShowdownTitleText);
    let underline = spawn_node(
        commands,
        root,
        Node {
            position_type: PositionType::Absolute,
            left: px(470),
            top: px(394),
            width: px(340),
            height: px(2),
            border_radius: BorderRadius::all(px(2)),
            ..default()
        },
        Some(Color::srgb(0.90, 0.72, 0.27).with_alpha(0.0)),
    );
    commands.entity(underline).insert((
        TexasShowdownUnderline,
        UiTransform {
            scale: Vec2::new(0.0, 1.0),
            ..UiTransform::IDENTITY
        },
        FocusPolicy::Pass,
    ));
}

fn add_texas_hand_result(
    commands: &mut Commands,
    table: Entity,
    game: &TexasHoldemSnapshot,
    assets: &UiAssets,
    avatars: &AvatarImages,
    animation: &GameSummaryAnimation,
) {
    let TexasHoldemPhaseView::HandComplete {
        showdown,
        awards,
        tournament_complete,
        reference_changes,
        ..
    } = &game.phase
    else {
        return;
    };
    let modal_visual = summary_modal_visual(animation.elapsed);
    let panel = spawn_node(
        commands,
        table,
        Node {
            position_type: PositionType::Absolute,
            left: percent(12),
            right: percent(12),
            top: percent(2),
            padding: UiRect::all(px(24)),
            flex_direction: FlexDirection::Column,
            row_gap: px(4),
            border_radius: BorderRadius::all(px(12)),
            ..default()
        },
        None,
    );
    commands.entity(panel).insert((
        GameSummaryModal,
        UiTransform::from_translation(Val2::px(0.0, modal_visual.offset_y)),
        GlobalZIndex(1200),
        FocusPolicy::Block,
        if animation.elapsed >= 0.0 {
            Visibility::Visible
        } else {
            Visibility::Hidden
        },
    ));
    let texture = decorate_panel_skin(commands, panel, PanelSkin::Window, assets);
    commands.entity(texture).insert(GameSummaryPanelTexture);
    add_animated_summary_text(
        commands,
        panel,
        "本手结算",
        25.0,
        ACCENT,
        0.0,
        animation.elapsed,
        assets,
    );

    let mut hand_players = game.players.iter().collect::<Vec<_>>();
    hand_players.sort_by(|left, right| {
        right
            .stack
            .cmp(&left.stack)
            .then_with(|| left.seat.0.cmp(&right.seat.0))
    });
    let hand_list = spawn_node(
        commands,
        panel,
        Node {
            width: percent(100),
            flex_direction: FlexDirection::Column,
            row_gap: px(3),
            ..default()
        },
        None,
    );
    for (index, player) in hand_players.iter().enumerate() {
        let delay = SUMMARY_ROW_START_DELAY + index as f32 * SUMMARY_ROW_INTERVAL;
        let row_progress = summary_row_progress(animation.elapsed, delay);
        let row = spawn_node(
            commands,
            hand_list,
            Node {
                width: percent(100),
                height: px(47),
                padding: UiRect::axes(px(9), px(3)),
                align_items: AlignItems::Center,
                column_gap: px(8),
                border_radius: BorderRadius::all(px(7)),
                ..default()
            },
            Some(PANEL_ALT.with_alpha(0.82 * row_progress)),
        );
        commands.entity(row).insert((
            GameSummaryRow { delay },
            UiTransform::from_translation(Val2::px(0.0, 12.0 * (1.0 - row_progress))),
            if row_progress > 0.0 {
                Visibility::Visible
            } else {
                Visibility::Hidden
            },
        ));
        add_texas_summary_avatar(commands, row, player, avatars, assets);
        let name = add_animated_summary_text(
            commands,
            row,
            &player.name,
            15.5,
            TEXT,
            delay,
            animation.elapsed,
            assets,
        );
        commands.entity(name).insert((
            Node {
                width: px(120),
                min_width: px(120),
                max_width: px(120),
                flex_shrink: 0.0,
                overflow: Overflow::clip(),
                ..default()
            },
            TextLayout::no_wrap(),
        ));
        let result = spawn_node(
            commands,
            row,
            Node {
                flex_grow: 1.0,
                min_width: px(0),
                align_items: AlignItems::Center,
                column_gap: px(7),
                ..default()
            },
            None,
        );
        let won_early = !*showdown
            && awards
                .iter()
                .any(|award| award.winners.contains(&player.id));
        if !*showdown {
            add_animated_summary_text(
                commands,
                result,
                if won_early { "胜利" } else { "弃牌" },
                18.0,
                if won_early { ACCENT } else { MUTED },
                delay,
                animation.elapsed,
                assets,
            );
        } else if player.folded {
            add_animated_summary_text(
                commands,
                result,
                "弃牌",
                18.0,
                MUTED,
                delay,
                animation.elapsed,
                assets,
            );
        } else if let Some(best) = game
            .revealed_hands
            .iter()
            .find(|revealed| revealed.player == player.id)
            .and_then(|revealed| revealed.best)
        {
            add_animated_summary_text(
                commands,
                result,
                texas_category_label(best.category()),
                14.0,
                READY,
                delay,
                animation.elapsed,
                assets,
            );
            for card in best.cards() {
                add_texas_card(commands, result, card, (28.0, 40.0), None, assets);
            }
        }
        let delta = i64::from(player.stack) - i64::from(player.hand_start_stack);
        add_animated_summary_text(
            commands,
            row,
            format!("{delta:+}"),
            19.0,
            if delta > 0 {
                READY
            } else if delta < 0 {
                DANGER
            } else {
                TEXT
            },
            delay,
            animation.elapsed,
            assets,
        );
    }

    let mut row_count = hand_players.len();
    if *tournament_complete {
        let divider_delay = SUMMARY_ROW_START_DELAY + row_count as f32 * SUMMARY_ROW_INTERVAL;
        let divider_progress = summary_row_progress(animation.elapsed, divider_delay);
        let divider = spawn_node(
            commands,
            panel,
            Node {
                width: percent(96),
                height: px(1),
                margin: UiRect::vertical(px(1)),
                align_self: AlignSelf::Center,
                ..default()
            },
            Some(Color::srgb(0.52, 0.55, 0.54).with_alpha(0.28 * divider_progress)),
        );
        commands.entity(divider).insert((
            GameSummaryDivider {
                delay: divider_delay,
            },
            if divider_progress > 0.0 {
                Visibility::Visible
            } else {
                Visibility::Hidden
            },
        ));
        let mut standings = hand_players.clone();
        standings.sort_by(|left, right| {
            right
                .stack
                .cmp(&left.stack)
                .then_with(|| left.seat.0.cmp(&right.seat.0))
        });
        for (index, player) in standings.iter().enumerate() {
            let delay = divider_delay + index as f32 * SUMMARY_ROW_INTERVAL;
            let progress = summary_row_progress(animation.elapsed, delay);
            let row = spawn_node(
                commands,
                panel,
                Node {
                    width: percent(100),
                    height: px(35),
                    padding: UiRect::axes(px(10), px(3)),
                    align_items: AlignItems::Center,
                    column_gap: px(8),
                    border_radius: BorderRadius::all(px(7)),
                    ..default()
                },
                Some(PANEL_ALT.with_alpha(0.82 * progress)),
            );
            commands.entity(row).insert((
                GameSummaryRow { delay },
                UiTransform::from_translation(Val2::px(0.0, 12.0 * (1.0 - progress))),
                if progress > 0.0 {
                    Visibility::Visible
                } else {
                    Visibility::Hidden
                },
            ));
            let avatar = player.avatar.and_then(|id| avatars.remote.get(&id));
            add_avatar(commands, row, &player.name, avatar, 26.0, assets);
            add_animated_summary_text(
                commands,
                row,
                format!("{}. {}", index + 1, player.name),
                14.5,
                TEXT,
                delay,
                animation.elapsed,
                assets,
            );
            let spacer = spawn_node(
                commands,
                row,
                Node {
                    flex_grow: 1.0,
                    ..default()
                },
                None,
            );
            commands.entity(spacer).insert(FocusPolicy::Pass);
            add_animated_summary_text(
                commands,
                row,
                format!("筹码 {}", player.stack),
                16.0,
                ACCENT,
                delay,
                animation.elapsed,
                assets,
            );
            if let Some(change) = reference_changes
                .iter()
                .find(|change| change.player == player.id)
            {
                add_animated_summary_text(
                    commands,
                    row,
                    format!("{:+}", change.delta),
                    15.0,
                    if change.delta >= 0 { READY } else { DANGER },
                    delay,
                    animation.elapsed,
                    assets,
                );
            }
        }
        row_count += standings.len();
    }

    let actions_delay = SUMMARY_ROW_START_DELAY
        + row_count as f32 * SUMMARY_ROW_INTERVAL
        + SUMMARY_ACTIONS_EXTRA_DELAY;
    let controls = spawn_node(
        commands,
        panel,
        Node {
            width: percent(100),
            height: px(46),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            column_gap: px(10),
            ..default()
        },
        None,
    );
    commands.entity(controls).insert((
        GameSummaryActions {
            delay: actions_delay,
        },
        if animation.elapsed >= actions_delay {
            Visibility::Visible
        } else {
            Visibility::Hidden
        },
    ));
    if *tournament_complete {
        add_action_button(
            commands,
            controls,
            "返回大厅",
            UiAction::ReturnToLobby,
            ButtonKind::Primary,
            assets,
        );
    } else {
        let ready = game
            .players
            .iter()
            .find(|player| player.id == game.you)
            .is_some_and(|player| player.ready);
        if ready {
            add_disabled_action_button(commands, controls, "已准备", assets);
        } else {
            add_action_button(
                commands,
                controls,
                "准备下一手",
                UiAction::PlayAgain,
                ButtonKind::Primary,
                assets,
            );
        }
    }
}

fn add_texas_summary_avatar(
    commands: &mut Commands,
    parent: Entity,
    player: &TexasHoldemPlayerState,
    avatars: &AvatarImages,
    assets: &UiAssets,
) {
    let ring = spawn_node(
        commands,
        parent,
        Node {
            width: px(38),
            height: px(38),
            min_width: px(38),
            position_type: PositionType::Relative,
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            border: UiRect::all(px(2)),
            border_radius: BorderRadius::all(percent(50)),
            ..default()
        },
        None,
    );
    commands
        .entity(ring)
        .insert(BorderColor::all(if player.ready {
            READY
        } else {
            MUTED.with_alpha(0.42)
        }));
    let avatar = player.avatar.and_then(|id| avatars.remote.get(&id));
    add_avatar(commands, ring, &player.name, avatar, 32.0, assets);
    if player.ready {
        let check = spawn_node(
            commands,
            ring,
            Node {
                position_type: PositionType::Absolute,
                right: px(-3),
                bottom: px(-2),
                width: px(15),
                height: px(15),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                border_radius: BorderRadius::all(percent(50)),
                ..default()
            },
            Some(READY),
        );
        add_text(commands, check, "✓", 10.0, Color::WHITE, assets);
    }
}

fn add_texas_card(
    commands: &mut Commands,
    parent: Entity,
    card: TexasHoldemCard,
    size: (f32, f32),
    animation: Option<(f32, Vec2)>,
    assets: &UiAssets,
) -> Entity {
    let face = texas_card_face(card, assets);
    let entity = commands
        .spawn((
            Node {
                width: px(size.0),
                height: px(size.1),
                border_radius: BorderRadius::all(px(5)),
                ..default()
            },
            ImageNode::new(face).with_color(if animation.is_some() {
                Color::WHITE.with_alpha(0.0)
            } else {
                Color::WHITE
            }),
            UiTransform::IDENTITY,
        ))
        .id();
    commands.entity(parent).add_child(entity);
    if let Some((delay, offset)) = animation {
        commands.entity(entity).insert(TexasDealCard {
            elapsed: 0.0,
            delay,
            offset,
        });
    }
    entity
}

pub(in crate::app) fn texas_card_face(card: TexasHoldemCard, assets: &UiAssets) -> Handle<Image> {
    assets
        .cards
        .get(&(qigui_rank(card.rank()), qigui_suit(card.suit())))
        .expect("德州普通牌面应当已随公共牌组加载")
        .clone()
}

pub(in crate::app) fn animate_texas_deal_cards(
    time: Res<Time>,
    mut cards: Query<(&mut TexasDealCard, &mut UiTransform, &mut ImageNode)>,
) {
    for (mut animation, mut transform, mut image) in &mut cards {
        animation.elapsed += time.delta_secs();
        let raw = ((animation.elapsed - animation.delay) / 0.28).clamp(0.0, 1.0);
        let movement = ease_out_cubic(raw);
        transform.translation = Val2::px(
            animation.offset.x * (1.0 - movement),
            animation.offset.y * (1.0 - movement),
        );
        transform.scale = Vec2::splat(0.76 + movement * 0.24);
        image.color = Color::WHITE.with_alpha(raw);
    }
}

pub(in crate::app) fn animate_texas_flying_card_backs(
    mut commands: Commands,
    time: Res<Time>,
    mut cards: Query<(
        Entity,
        &mut TexasFlyingCardBack,
        &mut Node,
        &mut UiTransform,
        &mut ImageNode,
    )>,
) {
    for (entity, mut animation, mut node, mut transform, mut image) in &mut cards {
        animation.elapsed += time.delta_secs();
        let raw = ((animation.elapsed - animation.delay) / 0.28).clamp(0.0, 1.0);
        if animation.elapsed < animation.delay {
            image.color = Color::WHITE.with_alpha(0.0);
            continue;
        }
        let movement = ease_out_cubic(raw);
        let position = animation.source.lerp(animation.target, movement);
        node.left = px(position.x);
        node.top = px(position.y);
        transform.rotation = Rot2::radians((1.0 - movement) * 0.08);
        transform.scale = Vec2::splat(0.88 + movement * 0.12);
        image.color = Color::WHITE.with_alpha((raw * 5.0).min(1.0));
        if raw >= 1.0 {
            commands.entity(entity).despawn();
        }
    }
}

pub(in crate::app) fn animate_texas_board_card_backs(
    time: Res<Time>,
    mut cards: Query<(&mut TexasBoardCardBack, &mut UiTransform, &mut ImageNode)>,
) {
    for (mut animation, mut transform, mut image) in &mut cards {
        animation.elapsed += time.delta_secs();
        let raw = ((animation.elapsed - animation.delay) / 0.25).clamp(0.0, 1.0);
        if animation.elapsed < animation.delay {
            image.color = Color::WHITE.with_alpha(0.0);
            transform.translation = Val2::px(animation.start_offset.x, animation.start_offset.y);
            continue;
        }
        let movement = ease_out_cubic(raw);
        let idle = (time.elapsed_secs() * 1.7 + animation.phase).sin();
        transform.translation = Val2::px(
            animation.start_offset.x * (1.0 - movement),
            animation.start_offset.y * (1.0 - movement) + idle * 0.8,
        );
        transform.rotation = Rot2::radians(idle * 0.006);
        transform.scale = Vec2::splat(0.90 + movement * 0.10);
        image.color = Color::WHITE.with_alpha(raw);
    }
}

pub(in crate::app) fn animate_texas_board_card_flips(
    time: Res<Time>,
    mut cards: Query<(&mut TexasBoardCardFlip, &mut UiTransform, &mut ImageNode)>,
) {
    for (mut animation, mut transform, mut image) in &mut cards {
        animation.elapsed += time.delta_secs();
        let progress = ((animation.elapsed - animation.delay) / 0.34).clamp(0.0, 1.0);
        if progress >= 0.5 && !animation.face_visible {
            image.image = animation.face.clone();
            animation.face_visible = true;
        }
        let horizontal = if progress < 0.5 {
            1.0 - ease_out_cubic(progress * 2.0)
        } else {
            ease_out_cubic((progress - 0.5) * 2.0)
        };
        transform.scale = Vec2::new(horizontal.max(0.025), 1.0 + (1.0 - horizontal) * 0.04);
    }
}

fn qigui_rank(rank: TexasRank) -> Rank {
    match rank {
        TexasRank::Two => Rank::Two,
        TexasRank::Three => Rank::Three,
        TexasRank::Four => Rank::Four,
        TexasRank::Five => Rank::Five,
        TexasRank::Six => Rank::Six,
        TexasRank::Seven => Rank::Seven,
        TexasRank::Eight => Rank::Eight,
        TexasRank::Nine => Rank::Nine,
        TexasRank::Ten => Rank::Ten,
        TexasRank::Jack => Rank::Jack,
        TexasRank::Queen => Rank::Queen,
        TexasRank::King => Rank::King,
        TexasRank::Ace => Rank::Ace,
    }
}

fn qigui_suit(suit: TexasSuit) -> Suit {
    match suit {
        TexasSuit::Diamond => Suit::Diamond,
        TexasSuit::Club => Suit::Club,
        TexasSuit::Heart => Suit::Heart,
        TexasSuit::Spade => Suit::Spade,
    }
}

fn texas_player_status(player: &TexasHoldemPlayerState) -> String {
    if !player.connected {
        "已离线".to_owned()
    } else if player.folded {
        "已弃牌".to_owned()
    } else if player.all_in {
        format!("全下 · 本轮 {}", player.committed_street)
    } else if player.committed_street > 0 {
        format!("本轮投入 {}", player.committed_street)
    } else {
        "等待行动".to_owned()
    }
}

fn texas_player_border_color(player: &TexasHoldemPlayerState, current: bool) -> Color {
    if player.folded {
        Color::srgb(0.48, 0.52, 0.50)
    } else if current {
        Color::NONE
    } else {
        READY
    }
}

fn street_label(phase: &TexasHoldemPhaseView) -> &'static str {
    match phase {
        TexasHoldemPhaseView::Betting {
            street: TexasStreet::PreFlop,
        } => "翻牌前",
        TexasHoldemPhaseView::Betting {
            street: TexasStreet::Flop,
        } => "翻牌圈",
        TexasHoldemPhaseView::Betting {
            street: TexasStreet::Turn,
        } => "转牌圈",
        TexasHoldemPhaseView::Betting {
            street: TexasStreet::River,
        } => "河牌圈",
        TexasHoldemPhaseView::HandComplete { .. } => "本手结算",
    }
}

fn texas_category_label(category: TexasHandCategory) -> &'static str {
    match category {
        TexasHandCategory::HighCard => "高牌",
        TexasHandCategory::OnePair => "一对",
        TexasHandCategory::TwoPair => "两对",
        TexasHandCategory::ThreeOfAKind => "三条",
        TexasHandCategory::Straight => "顺子",
        TexasHandCategory::Flush => "同花",
        TexasHandCategory::FullHouse => "葫芦",
        TexasHandCategory::FourOfAKind => "四条",
        TexasHandCategory::StraightFlush => "同花顺",
        TexasHandCategory::RoyalFlush => "皇家同花顺",
    }
}
