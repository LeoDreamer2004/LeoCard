use super::*;

const SHENGJI_PLAYER_COUNT: u8 = 4;
/// 每位玩家相邻两张可见手牌的发牌间隔；对应全桌约 100ms 发一张牌。
const SHENGJI_LOCAL_DEAL_INTERVAL: f32 = 0.40;

#[derive(Component)]
pub(in crate::app) struct ShengjiBiddingCountdown;

pub(in crate::app) fn shengji_bidding_countdown_label(milliseconds: u16) -> String {
    format!("{}秒", milliseconds.div_ceil(1000))
}

pub(in crate::app) fn sync_shengji_bidding_countdown(
    client: Option<Res<ClientResource>>,
    mut labels: Query<&mut Text, With<ShengjiBiddingCountdown>>,
) {
    let Some(milliseconds) = client
        .as_deref()
        .and_then(|client| client.0.model().shengji_game())
        .and_then(|game| match game.phase {
            ShengjiPhaseView::BiddingGrace {
                milliseconds_remaining,
                ..
            } => Some(milliseconds_remaining),
            _ => None,
        })
    else {
        return;
    };
    let expected = shengji_bidding_countdown_label(milliseconds);
    for mut label in &mut labels {
        if label.0 != expected {
            label.0.clone_from(&expected);
        }
    }
}

pub(in crate::app) struct ShengjiTableVisuals<'a> {
    pub(in crate::app) assets: &'a UiAssets,
    pub(in crate::app) avatars: &'a AvatarImages,
    pub(in crate::app) appearance: &'a TableAppearance,
    pub(in crate::app) brightness: f32,
    pub(in crate::app) vignette: f32,
    pub(in crate::app) table_materials: &'a mut Assets<TableBackgroundMaterial>,
    pub(in crate::app) turn_border_materials: &'a mut Assets<TurnBorderMaterial>,
    pub(in crate::app) start_game_transition: &'a StartGameSeatTransition,
    pub(in crate::app) score_capture: &'a ShengjiScoreCaptureEffectState,
    pub(in crate::app) settlement: &'a ShengjiSettlementAnimation,
    pub(in crate::app) presentation: &'a ShengjiPresentationState,
}

pub(in crate::app) fn render_shengji_table(
    commands: &mut Commands,
    root: Entity,
    client: &ClientResource,
    game: &ShengjiSnapshot,
    ui: &mut UiState,
    chat: &ChatPanelState,
    visuals: ShengjiTableVisuals,
) {
    if ui.shengji_observed_match != Some(game.match_id)
        || ui.shengji_observed_hand_number != game.hand_number
    {
        ui.shengji_observed_match = Some(game.match_id);
        ui.shengji_observed_hand_number = game.hand_number;
        ui.selected_shengji.clear();
        ui.shengji_buried_open = false;
    }
    if game.your_buried.is_empty() {
        ui.shengji_buried_open = false;
    }
    ui.selected_shengji
        .retain(|card| game.your_hand.contains(card));
    select_forced_shengji_follow_cards(game, ui);
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
    let felt = visuals
        .appearance
        .custom_felt
        .as_ref()
        .unwrap_or(&visuals.assets.table_felt)
        .clone();
    let material = visuals.table_materials.add(TableBackgroundMaterial {
        params: table_material_params(
            visuals.brightness,
            visuals.vignette,
            visuals.appearance.custom_felt.is_none(),
        ),
        texture: felt,
    });
    commands
        .entity(content)
        .insert((MaterialNode(material), TableBackground));

    let table = spawn_node(
        commands,
        content,
        Node {
            width: percent(100),
            flex_grow: 1.0,
            min_height: px(410),
            position_type: PositionType::Relative,
            ..default()
        },
        None,
    );
    if let NetworkState::Reconnecting(message) = client.0.state() {
        add_reconnecting_overlay(commands, table, message, visuals.assets);
    }

    let own = game
        .players
        .iter()
        .find(|player| player.id == game.you)
        .expect("双升快照包含接收者");
    let start_transition_active = visuals.start_game_transition.is_active_for(game.match_id);
    let previous_trick = visuals.presentation.revealed_previous_trick();
    for relative in 1..SHENGJI_PLAYER_COUNT {
        let seat = SeatId((own.seat.0 + relative) % SHENGJI_PLAYER_COUNT);
        if let Some(player) = game.players.iter().find(|player| player.seat == seat) {
            add_shengji_opponent(
                commands,
                table,
                relative,
                player,
                game,
                visuals.assets,
                visuals.avatars,
                ui.interaction_menu_open,
                visuals.turn_border_materials,
                previous_trick,
                start_transition_active,
            );
        }
    }
    let bidding_visible = matches!(
        game.phase,
        ShengjiPhaseView::Dealing { .. } | ShengjiPhaseView::BiddingGrace { .. }
    );
    add_shengji_own_play(commands, table, game, visuals.assets, previous_trick);
    add_shengji_collecting_tray(
        commands,
        table,
        game,
        client.0.model().shengji_collected_score_cards(),
        visuals.score_capture,
        visuals.assets,
    );
    add_shengji_throw_penalty_effect(commands, table, game, visuals.assets);
    if let ShengjiPhaseView::BottomFlipping { reveal } = &game.phase {
        add_shengji_bottom_flip(commands, table, game, reveal.as_ref(), visuals.assets);
    }
    add_shengji_presentation_overlay(commands, table, game, visuals.presentation, visuals.assets);

    let finished = matches!(game.phase, ShengjiPhaseView::Finished { .. });
    if ui.shengji_buried_open && !finished {
        add_shengji_private_buried(commands, table, game, visuals.assets);
    }
    if !finished {
        let hand_area = spawn_node(
            commands,
            content,
            Node {
                width: percent(100),
                height: px(180),
                flex_shrink: 0.0,
                position_type: PositionType::Relative,
                ..default()
            },
            None,
        );
        if bidding_visible {
            add_shengji_bidding_panel(commands, hand_area, game, visuals.assets);
        }
        add_shengji_actions(commands, hand_area, game, ui, visuals.assets);
        add_shengji_hand(commands, hand_area, game, ui, visuals.assets);
        add_shengji_self_panel(
            commands,
            hand_area,
            own,
            game,
            visuals.assets,
            visuals.avatars,
            visuals.turn_border_materials,
            start_transition_active,
        );
    }
    let local_auto_play = matches!(
        game.phase,
        ShengjiPhaseView::Burying
            | ShengjiPhaseView::BottomCopying { .. }
            | ShengjiPhaseView::BottomCopyBurying { .. }
            | ShengjiPhaseView::FiveTrumpCrossing { .. }
            | ShengjiPhaseView::Playing
    )
    .then_some(own.auto_play);
    add_chat_panel(
        commands,
        content,
        chat,
        visuals.assets,
        local_auto_play,
        shengji_previous_trick_button_state(&game.phase, visuals.presentation.has_previous_trick()),
        (!game.your_buried.is_empty() && !finished).then_some(true),
    );
    if local_auto_play == Some(true) {
        add_auto_play_overlay(commands, content, visuals.assets);
    }

    if let ShengjiPhaseView::Finished { result, buried } = &game.phase {
        add_shengji_result(
            commands,
            table,
            game,
            result,
            buried,
            visuals.assets,
            visuals.avatars,
            visuals.settlement,
        );
    }
}

pub(in crate::app) fn shengji_previous_trick_button_state(
    phase: &ShengjiPhaseView,
    has_previous_trick: bool,
) -> Option<bool> {
    matches!(phase, ShengjiPhaseView::Playing).then_some(has_previous_trick)
}

fn add_shengji_private_buried(
    commands: &mut Commands,
    table: Entity,
    game: &ShengjiSnapshot,
    assets: &UiAssets,
) {
    let backdrop = spawn_node(
        commands,
        table,
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
        Some(Color::BLACK.with_alpha(0.48)),
    );
    commands
        .entity(backdrop)
        .insert((Button, UiAction::ToggleShengjiBuried, GlobalZIndex(1750)));

    let panel = spawn_node(
        commands,
        backdrop,
        Node {
            min_width: px(430),
            min_height: px(184),
            padding: UiRect::axes(px(28), px(22)),
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            row_gap: px(11),
            border_radius: BorderRadius::all(px(8)),
            ..default()
        },
        Some(HEADER_BG.with_alpha(0.98)),
    );
    commands.entity(panel).insert((
        BorderColor::all(BORDER),
        BoxShadow::new(Color::BLACK.with_alpha(0.55), px(2), px(8), px(0), px(15)),
        FocusPolicy::Pass,
    ));
    decorate_panel_skin(commands, panel, PanelSkin::Popup, assets);
    add_text(commands, panel, "我的底牌", 22.0, ACCENT, assets);
    add_shengji_card_row(
        commands,
        panel,
        &game.your_buried,
        ShengjiCardSize::Seat,
        shengji_display_trump(game),
        assets,
    );
    add_text(
        commands,
        panel,
        "仅你可见 · 点击任意位置收起",
        13.0,
        MUTED,
        assets,
    );
}

fn add_shengji_bottom_flip(
    commands: &mut Commands,
    table: Entity,
    game: &ShengjiSnapshot,
    reveal: Option<&leocard_protocol::ShengjiBottomFlipRevealView>,
    assets: &UiAssets,
) {
    let panel = spawn_node(
        commands,
        table,
        Node {
            position_type: PositionType::Absolute,
            left: percent(31),
            right: percent(31),
            top: percent(18),
            min_height: px(210),
            padding: UiRect::all(px(20)),
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            row_gap: px(9),
            ..default()
        },
        Some(HEADER_BG.with_alpha(0.94)),
    );
    commands.entity(panel).insert((
        GlobalZIndex(900),
        FocusPolicy::Pass,
        BoxShadow::new(Color::BLACK.with_alpha(0.45), px(2), px(7), px(0), px(12)),
    ));
    decorate_panel_skin(commands, panel, PanelSkin::Section, assets);
    add_text(commands, panel, "扳底", 22.0, ACCENT, assets);

    let Some(reveal) = reveal else {
        add_text(commands, panel, "准备翻开底牌…", 15.0, MUTED, assets);
        return;
    };
    let central_card = add_shengji_card_row(
        commands,
        panel,
        &[reveal.card],
        ShengjiCardSize::Seat,
        shengji_display_trump(game),
        assets,
    );
    commands.entity(central_card).insert((
        ShengjiBottomFlipPanelElement::CentralCard,
        UiTransform::IDENTITY,
        Visibility::Visible,
    ));
    if reveal.matches.is_empty() {
        add_text(
            commands,
            panel,
            "无人持有同牌，继续翻牌",
            14.0,
            MUTED,
            assets,
        );
    } else {
        for (index, matched) in reveal.matches.iter().enumerate() {
            let row = spawn_node(
                commands,
                panel,
                Node {
                    min_height: px(52),
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    column_gap: px(10),
                    ..default()
                },
                None,
            );
            commands.entity(row).insert((
                ShengjiBottomFlipPanelElement::MatchRow {
                    player: matched.player,
                    index,
                    count: reveal.matches.len(),
                },
                UiTransform::IDENTITY,
                Visibility::Visible,
            ));
            let name = game
                .players
                .iter()
                .find(|player| player.id == matched.player)
                .map_or("玩家", |player| player.name.as_str());
            add_text(commands, row, name, 15.0, TEXT, assets);
            add_shengji_card_row(
                commands,
                row,
                &matched.cards,
                ShengjiCardSize::Score,
                shengji_display_trump(game),
                assets,
            );
        }
    }
    if let Some(dealer) = reveal.dealer {
        let name = game
            .players
            .iter()
            .find(|player| player.id == dealer)
            .map_or("该玩家", |player| player.name.as_str());
        let resolution = add_text(
            commands,
            panel,
            &format!("{name} 坐庄"),
            18.0,
            ACCENT,
            assets,
        );
        commands.entity(resolution).insert((
            ShengjiBottomFlipPanelElement::DealerLine,
            UiTransform::IDENTITY,
            Visibility::Visible,
        ));
    } else {
        let resolution = add_text(commands, panel, "尚未定庄，继续翻牌", 14.0, MUTED, assets);
        commands.entity(resolution).insert((
            ShengjiBottomFlipPanelElement::DealerLine,
            UiTransform::IDENTITY,
            Visibility::Visible,
        ));
    }
}

fn add_shengji_opponent(
    commands: &mut Commands,
    table: Entity,
    relative: u8,
    player: &ShengjiPlayerState,
    game: &ShengjiSnapshot,
    assets: &UiAssets,
    avatars: &AvatarImages,
    interaction_menu_open: Option<PlayerId>,
    turn_border_materials: &mut Assets<TurnBorderMaterial>,
    previous_trick: Option<&[ShengjiPublicPlay]>,
    start_transition_active: bool,
) {
    // 机器人图标会从人物框向牌桌内侧伸出 38px；左右出牌区额外留白，
    // 保证牌、机器人标记和人物框各自拥有清晰的视觉边界。
    const SIDE_PLAY_GAP: f32 = 68.0;
    const SIDE_SLOT_WIDTH: f32 = 188.0 + SIDE_PLAY_GAP + 165.0;
    let side = match relative {
        1 => SeatSide::Left,
        2 => SeatSide::Top,
        3 => SeatSide::Right,
        _ => unreachable!(),
    };
    let mut node = Node {
        position_type: PositionType::Absolute,
        align_items: AlignItems::Center,
        column_gap: px(if matches!(side, SeatSide::Top) {
            12.0
        } else {
            SIDE_PLAY_GAP
        }),
        row_gap: px(7),
        ..default()
    };
    match side {
        SeatSide::Left => {
            node.left = px(12);
            node.top = percent(50);
            node.width = px(SIDE_SLOT_WIDTH);
        }
        SeatSide::Top => {
            node.left = percent(32);
            node.right = percent(32);
            node.top = px(12);
            node.flex_direction = FlexDirection::Column;
        }
        SeatSide::Right => {
            node.right = px(12);
            node.top = percent(50);
            node.width = px(SIDE_SLOT_WIDTH);
            node.justify_content = JustifyContent::FlexEnd;
        }
    }
    let slot = spawn_node(commands, table, node, None);
    if !matches!(side, SeatSide::Top) {
        // 百分比定位以节点上沿为基准；回拉半个出牌槽高度，使左右人物框
        // 在任意窗口比例下都真正落在牌桌垂直中线上。
        commands
            .entity(slot)
            .insert(UiTransform::from_translation(Val2::px(0.0, -52.0)));
    }
    if matches!(side, SeatSide::Right) {
        add_shengji_play_area(
            commands,
            slot,
            game,
            player.id,
            side,
            assets,
            previous_trick,
        );
    }
    add_shengji_player_panel(
        commands,
        slot,
        player,
        game,
        side,
        assets,
        avatars,
        interaction_menu_open,
        turn_border_materials,
        start_transition_active,
    );
    if !matches!(side, SeatSide::Right) {
        add_shengji_play_area(
            commands,
            slot,
            game,
            player.id,
            side,
            assets,
            previous_trick,
        );
    }
}

fn add_shengji_player_panel(
    commands: &mut Commands,
    parent: Entity,
    player: &ShengjiPlayerState,
    game: &ShengjiSnapshot,
    side: SeatSide,
    assets: &UiAssets,
    avatars: &AvatarImages,
    interaction_menu_open: Option<PlayerId>,
    turn_border_materials: &mut Assets<TurnBorderMaterial>,
    start_transition_active: bool,
) {
    let current = game.current_player == Some(player.id);
    let panel = spawn_node(
        commands,
        parent,
        Node {
            width: px(188),
            min_width: px(188),
            height: px(72),
            min_height: px(72),
            // 双升人物框没有七鬼/德州的内侧大数值区，使用完全对称的
            // padding，确保“头像 + 文字”组合在三个方向上都处于框中心。
            padding: UiRect::axes(px(8), px(4)),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            column_gap: px(8),
            border: UiRect::all(px(1)),
            border_radius: BorderRadius::all(px(8)),
            ..default()
        },
        Some(HEADER_BG),
    );
    commands.entity(panel).insert((
        BorderColor::all(BORDER),
        Button,
        UiAction::ToggleInteractionMenu(player.id),
    ));
    attach_start_game_seat_transition(commands, panel, player.id, start_transition_active);
    decorate_player_panel(commands, panel, assets, 1.0);
    if current {
        add_turn_border_trace(
            commands,
            panel,
            turn_border_materials,
            TurnBorderAnimationKey::new(GameKind::Shengji, game.match_id, player.id),
        );
    }
    let avatar = player.avatar.and_then(|id| avatars.remote.get(&id));
    let avatar_entity = add_avatar(commands, panel, &player.name, avatar, 34.0, assets);
    commands
        .entity(avatar_entity)
        .insert(PlayerAvatarAnchor(player.id));
    if player.auto_play {
        add_auto_play_robot_indicator(commands, panel, player.id, side, assets);
    }
    let details = spawn_node(
        commands,
        panel,
        Node {
            min_width: px(82),
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            ..default()
        },
        None,
    );
    add_text(commands, details, &player.name, 14.0, TEXT, assets);
    let level = add_text(
        commands,
        details,
        format!(
            "{} · {}张",
            shengji_level_label(game.levels[usize::from(player.id.0 % 2)]),
            player.hand_len
        ),
        11.0,
        MUTED,
        assets,
    );
    commands
        .entity(level)
        .insert(ShengjiLevelIndicator { base_color: MUTED });
    if game.dealer == Some(player.id) {
        let dealer = spawn_node(
            commands,
            panel,
            Node {
                position_type: PositionType::Absolute,
                right: if matches!(side, SeatSide::Right) {
                    Val::Auto
                } else {
                    px(5)
                },
                left: if matches!(side, SeatSide::Right) {
                    px(5)
                } else {
                    Val::Auto
                },
                top: px(4),
                width: px(24),
                height: px(24),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                border_radius: BorderRadius::all(percent(50)),
                ..default()
            },
            Some(ACCENT),
        );
        commands.entity(dealer).insert(ShengjiDealerBadge);
        add_text(commands, dealer, "庄", 12.0, Color::BLACK, assets);
    }
    let interaction_menu = add_interaction_menu(
        commands,
        panel,
        player.id,
        side,
        &player.name,
        avatar,
        player.reference_points,
        player.completed_games,
        &player.game_profiles,
        assets,
    );
    commands
        .entity(interaction_menu)
        .insert(if interaction_menu_open == Some(player.id) {
            Visibility::Visible
        } else {
            Visibility::Hidden
        });
    commands.entity(panel).insert(OpponentBadge {
        player: player.id,
        score_popup: None,
        interaction_menu,
    });
}

fn add_shengji_bidding_panel(
    commands: &mut Commands,
    hand_area: Entity,
    game: &ShengjiSnapshot,
    assets: &UiAssets,
) {
    let panel = spawn_node(
        commands,
        hand_area,
        Node {
            position_type: PositionType::Absolute,
            left: percent(29),
            right: percent(29),
            top: px(2),
            height: px(34),
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            column_gap: px(6),
            ..default()
        },
        None,
    );
    commands.entity(panel).insert(GlobalZIndex(80));
    for (label, target, color) in [
        ("♦ 方块", Some(ShengjiSuit::Diamond), DANGER),
        ("♣ 梅花", Some(ShengjiSuit::Club), TEXT),
        ("♥ 红桃", Some(ShengjiSuit::Heart), DANGER),
        ("♠ 黑桃", Some(ShengjiSuit::Spade), TEXT),
        ("无主", None, ACCENT),
    ] {
        let cards = shengji_declaration_candidate(game, target);
        add_shengji_bid_button(commands, panel, label, cards, color, false, assets);
    }
    if let ShengjiPhaseView::BiddingGrace {
        milliseconds_remaining,
        confirmed_count,
        you_confirmed,
        ..
    } = &game.phase
    {
        let countdown = add_text(
            commands,
            panel,
            shengji_bidding_countdown_label(*milliseconds_remaining),
            13.0,
            ACCENT,
            assets,
        );
        commands.entity(countdown).insert(ShengjiBiddingCountdown);
        let pass_label = if *you_confirmed {
            format!("✓ 已确认 {confirmed_count}/4")
        } else {
            let action = match game.declaration.as_ref() {
                None => "不亮主",
                Some(declaration) if declaration.player == game.you => "不加亮",
                Some(_) => "不反主",
            };
            format!("{action} {confirmed_count}/4")
        };
        add_shengji_bid_pass_button(commands, panel, &pass_label, *you_confirmed, assets);
    }
}

fn add_shengji_bid_pass_button(
    commands: &mut Commands,
    parent: Entity,
    label: &str,
    confirmed: bool,
    assets: &UiAssets,
) {
    let normal = if confirmed {
        Color::srgb(0.12, 0.31, 0.22)
    } else {
        Color::srgb(0.16, 0.43, 0.29)
    };
    let mut entity = commands.spawn((
        Node {
            min_width: px(88),
            height: px(34),
            padding: UiRect::axes(px(8), px(4)),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            ..default()
        },
        ImageNode::new(assets.primary_button.clone())
            .with_mode(NodeImageMode::Stretch)
            .with_color(normal),
    ));
    if !confirmed {
        entity.insert((
            Button,
            UiAction::ConfirmShengjiBidPass,
            ButtonTint {
                normal,
                hovered: Color::srgb(0.23, 0.58, 0.39),
                pressed: Color::srgb(0.10, 0.30, 0.20),
            },
        ));
    }
    let entity = entity.id();
    commands.entity(parent).add_child(entity);
    add_text(
        commands,
        entity,
        label,
        12.0,
        if confirmed { MUTED } else { Color::WHITE },
        assets,
    );
}

fn add_shengji_bid_button(
    commands: &mut Commands,
    parent: Entity,
    label: &str,
    cards: Option<Vec<ShengjiCard>>,
    color: Color,
    bottom_copy: bool,
    assets: &UiAssets,
) {
    let enabled = cards.is_some();
    let entity = commands
        .spawn((
            Node {
                min_width: px(66),
                height: px(34),
                padding: UiRect::axes(px(6), px(4)),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
            ImageNode::new(if enabled {
                assets.primary_button.clone()
            } else {
                assets.disabled_button.clone()
            })
            .with_mode(NodeImageMode::Stretch)
            .with_color(if enabled {
                Color::srgb(0.20, 0.62, 0.38)
            } else {
                Color::srgb(0.35, 0.37, 0.37)
            }),
        ))
        .id();
    commands.entity(parent).add_child(entity);
    if let Some(cards) = cards {
        commands.entity(entity).insert((
            Button,
            if bottom_copy {
                UiAction::ShengjiBottomCopy(cards)
            } else {
                UiAction::ShengjiDeclare(cards)
            },
            ButtonTint {
                normal: Color::srgb(0.20, 0.62, 0.38),
                hovered: Color::srgb(0.27, 0.75, 0.47),
                pressed: Color::srgb(0.14, 0.46, 0.28),
            },
        ));
    }
    add_text(
        commands,
        entity,
        label,
        13.0,
        if enabled {
            color
        } else {
            Color::srgb(0.58, 0.60, 0.60)
        },
        assets,
    );
}

pub(in crate::app) fn shengji_declaration_candidate(
    game: &ShengjiSnapshot,
    suit: Option<ShengjiSuit>,
) -> Option<Vec<ShengjiCard>> {
    let level = shengji_current_level(game);
    if game.rules.bid_with_joker {
        return joker_bid_declaration_candidate(game, suit, level);
    }
    let exposed = game
        .declaration
        .as_ref()
        .map_or(&[][..], |declaration| declaration.cards.as_slice());
    let available = game
        .your_hand
        .iter()
        .copied()
        .filter(|card| !exposed.contains(card))
        .collect::<Vec<_>>();
    if game.rules.deck_count >= 3 {
        return multi_deck_declaration_candidate(game, suit, level, &available);
    }
    match suit {
        Some(suit) => {
            let matching = available
                .iter()
                .copied()
                .filter(|card| card.rank() == level && card.suit() == Some(suit))
                .collect::<Vec<_>>();
            let protecting = game.declaration.as_ref().is_some_and(|declaration| {
                declaration.player == game.you
                    && declaration.cards.len() == 1
                    && declaration.trump == ShengjiBidTrump::Suit(suit)
            });
            if game.declaration.is_none() || protecting {
                matching.first().copied().map(|card| vec![card])
            } else if matching.len() >= 2
                && game.declaration.as_ref().is_some_and(|declaration| {
                    !declaration.protected
                        && ShengjiBidTrump::Suit(suit).strength() > declaration.trump.strength()
                })
            {
                Some(matching[..2].to_vec())
            } else {
                None
            }
        }
        None => [ShengjiRank::BigJoker, ShengjiRank::SmallJoker]
            .into_iter()
            .find_map(|rank| {
                let matching = available
                    .iter()
                    .copied()
                    .filter(|card| card.rank() == rank)
                    .collect::<Vec<_>>();
                let trump = if rank == ShengjiRank::BigJoker {
                    ShengjiBidTrump::NoTrumpBigJoker
                } else {
                    ShengjiBidTrump::NoTrumpSmallJoker
                };
                (matching.len() >= 2
                    && game
                        .declaration
                        .as_ref()
                        .is_none_or(|declaration| trump.strength() > declaration.trump.strength()))
                .then(|| matching[..2].to_vec())
            }),
    }
}

fn joker_bid_declaration_candidate(
    game: &ShengjiSnapshot,
    suit: Option<ShengjiSuit>,
    level: ShengjiRank,
) -> Option<Vec<ShengjiCard>> {
    let current = game.declaration.as_ref();
    // 带王亮时只有王能由本人复用；过去公开过的级牌必须排除。私人快照
    // 会在重连后继续提供这组牌，避免按钮生成服务端必然拒绝的候选。
    let unexposed_primary = game
        .your_hand
        .iter()
        .copied()
        .filter(|card| !game.your_exposed_cards.contains(card))
        .collect::<Vec<_>>();
    let current_count = current.map(|declaration| declaration_primary_count(declaration, level));
    let current_strength = current.and_then(|declaration| {
        declaration
            .trump
            .declaration_strength(game.rules.deck_count, current_count.unwrap_or_default())
    });
    let legal_strength = |trump: ShengjiBidTrump, count: usize| {
        let strength = trump.declaration_strength(game.rules.deck_count, count)?;
        if let Some(declaration) = current {
            if declaration.protected
                && matches!(trump, ShengjiBidTrump::Suit(_))
                && count <= current_count.unwrap_or_default()
            {
                return None;
            }
            if current_strength.is_some_and(|current| strength <= current) {
                return None;
            }
        }
        Some(strength)
    };

    match suit {
        Some(suit) => {
            let trump = ShengjiBidTrump::Suit(suit);
            let matching = unexposed_primary
                .iter()
                .copied()
                .filter(|card| card.rank() == level && card.suit() == Some(suit))
                .collect::<Vec<_>>();
            let companion_rank = shengji_bid_joker_for_suit(suit);
            let extends_current = current.is_some_and(|declaration| {
                declaration.player == game.you && declaration.trump == trump
            });
            let companion = current
                .filter(|_| extends_current)
                .and_then(|declaration| {
                    declaration
                        .cards
                        .iter()
                        .copied()
                        .find(|card| card.rank() == companion_rank && card.suit().is_none())
                })
                .or_else(|| {
                    game.your_hand
                        .iter()
                        .copied()
                        .find(|card| card.rank() == companion_rank && card.suit().is_none())
                })?;
            let existing_count = extends_current
                .then(|| current_count.unwrap_or_default())
                .unwrap_or_default();
            let minimum_count = if current.is_none() {
                1
            } else if extends_current {
                existing_count + 1
            } else {
                2
            };
            for total_count in minimum_count..=usize::from(game.rules.deck_count) {
                if legal_strength(trump, total_count).is_none() {
                    continue;
                }
                let needed = total_count.saturating_sub(existing_count);
                if matching.len() < needed {
                    continue;
                }
                let mut cards = matching[..needed].to_vec();
                let current_has_companion = extends_current
                    && current.is_some_and(|declaration| {
                        declaration.cards.iter().any(|card| *card == companion)
                    });
                if !current_has_companion {
                    cards.push(companion);
                }
                return Some(cards);
            }
            None
        }
        None => {
            // 无主在带王亮中只能反主，不能作为首次亮牌。
            current?;
            let mut candidates = Vec::<(u8, Vec<ShengjiCard>)>::new();
            for (trump, rank) in [
                (ShengjiBidTrump::NoTrumpSmallJoker, ShengjiRank::SmallJoker),
                (ShengjiBidTrump::NoTrumpBigJoker, ShengjiRank::BigJoker),
            ] {
                let matching = game
                    .your_hand
                    .iter()
                    .copied()
                    .filter(|card| card.rank() == rank)
                    .collect::<Vec<_>>();
                let extends_current = current.is_some_and(|declaration| {
                    declaration.player == game.you && declaration.trump == trump
                });
                let existing_cards = current
                    .filter(|_| extends_current)
                    .map_or(&[][..], |declaration| declaration.cards.as_slice());
                let available = matching
                    .iter()
                    .copied()
                    .filter(|card| !existing_cards.contains(card))
                    .collect::<Vec<_>>();
                let existing_count = existing_cards.len();
                let minimum_count = if extends_current {
                    (existing_count + 1).max(2)
                } else {
                    2
                };
                for total_count in minimum_count..=usize::from(game.rules.deck_count) {
                    let Some(strength) = legal_strength(trump, total_count) else {
                        continue;
                    };
                    let needed = total_count.saturating_sub(existing_count);
                    if available.len() >= needed {
                        candidates.push((strength, available[..needed].to_vec()));
                    }
                }
            }
            candidates.sort_by_key(|(strength, _)| *strength);
            candidates.into_iter().next().map(|(_, cards)| cards)
        }
    }
}

fn declaration_primary_count(
    declaration: &leocard_protocol::ShengjiDeclarationView,
    level: ShengjiRank,
) -> usize {
    match declaration.trump {
        ShengjiBidTrump::Suit(suit) => declaration
            .cards
            .iter()
            .filter(|card| card.rank() == level && card.suit() == Some(suit))
            .count(),
        ShengjiBidTrump::NoTrumpSmallJoker => declaration
            .cards
            .iter()
            .filter(|card| card.rank() == ShengjiRank::SmallJoker)
            .count(),
        ShengjiBidTrump::NoTrumpBigJoker => declaration
            .cards
            .iter()
            .filter(|card| card.rank() == ShengjiRank::BigJoker)
            .count(),
    }
}

fn multi_deck_declaration_candidate(
    game: &ShengjiSnapshot,
    suit: Option<ShengjiSuit>,
    level: ShengjiRank,
    available: &[ShengjiCard],
) -> Option<Vec<ShengjiCard>> {
    let current_strength = game.declaration.as_ref().and_then(|declaration| {
        declaration
            .trump
            .declaration_strength(game.rules.deck_count, declaration.cards.len())
    });
    let mut candidates = Vec::<(u8, Vec<ShengjiCard>)>::new();
    let faces = match suit {
        Some(suit) => vec![(ShengjiBidTrump::Suit(suit), Some(suit), level)],
        None => vec![
            (
                ShengjiBidTrump::NoTrumpSmallJoker,
                None,
                ShengjiRank::SmallJoker,
            ),
            (
                ShengjiBidTrump::NoTrumpBigJoker,
                None,
                ShengjiRank::BigJoker,
            ),
        ],
    };
    for (trump, face_suit, rank) in faces {
        let matching = available
            .iter()
            .copied()
            .filter(|card| card.rank() == rank && card.suit() == face_suit)
            .collect::<Vec<_>>();
        let minimum_count = if game.declaration.is_some() {
            2
        } else {
            usize::from(face_suit.is_none()) + 1
        };
        for total_count in minimum_count..=usize::from(game.rules.deck_count) {
            let Some(strength) = trump.declaration_strength(game.rules.deck_count, total_count)
            else {
                continue;
            };
            if current_strength.is_some_and(|current| strength <= current) {
                continue;
            }
            let existing_count = game.declaration.as_ref().map_or(0, |declaration| {
                usize::from(
                    declaration.player == game.you
                        && declaration.trump == trump
                        && declaration
                            .cards
                            .iter()
                            .all(|card| card.rank() == rank && card.suit() == face_suit),
                ) * declaration.cards.len()
            });
            if total_count <= existing_count {
                continue;
            }
            let needed = total_count - existing_count;
            if matching.len() >= needed {
                candidates.push((strength, matching[..needed].to_vec()));
            }
        }
    }
    candidates.sort_by_key(|(strength, _)| *strength);
    candidates.into_iter().next().map(|(_, cards)| cards)
}

fn add_shengji_play_area(
    commands: &mut Commands,
    parent: Entity,
    game: &ShengjiSnapshot,
    player: PlayerId,
    side: SeatSide,
    assets: &UiAssets,
    previous_trick: Option<&[ShengjiPublicPlay]>,
) {
    let area = spawn_node(
        commands,
        parent,
        Node {
            width: px(165),
            min_width: px(165),
            min_height: px(104),
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::Center,
            justify_content: match side {
                SeatSide::Left => JustifyContent::FlexStart,
                SeatSide::Top => JustifyContent::Center,
                SeatSide::Right => JustifyContent::FlexEnd,
            },
            ..default()
        },
        None,
    );
    let throw_failure = previous_trick.is_none().then_some(()).and_then(|_| {
        game.throw_failure
            .as_ref()
            .filter(|failure| failure.player == player)
    });
    let cards = if let Some(previous_trick) = previous_trick {
        previous_trick
            .iter()
            .find(|played| played.player == player)
            .map(|played| played.play.cards.as_slice())
    } else if matches!(
        game.phase,
        ShengjiPhaseView::Dealing { .. }
            | ShengjiPhaseView::BiddingGrace { .. }
            | ShengjiPhaseView::BottomCopyBurying { .. }
    ) {
        game.declaration
            .as_ref()
            .filter(|declaration| declaration.player == player)
            .map(|declaration| declaration.cards.as_slice())
    } else if let Some(failure) = throw_failure {
        Some(failure.attempted.as_slice())
    } else {
        game.trick.as_ref().and_then(|trick| {
            trick
                .plays
                .iter()
                .find(|played| played.player == player)
                .map(|played| played.play.cards.as_slice())
        })
    };
    if let Some(cards) = cards {
        if let Some(failure) = throw_failure {
            let holder = spawn_node(
                commands,
                area,
                Node {
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    row_gap: px(2),
                    ..default()
                },
                None,
            );
            commands
                .entity(holder)
                .insert((UiTransform::IDENTITY, Visibility::Visible));
            let label = add_text(commands, holder, "甩牌失败", 15.0, DANGER, assets);
            commands.entity(label).insert((
                ShengjiFailedThrowLabel {
                    returning: failure.stage == ShengjiThrowFailureStage::Returning,
                    elapsed: 0.0,
                },
                TextShadow {
                    offset: Vec2::new(1.0, 1.0),
                    color: Color::BLACK.with_alpha(0.88),
                },
            ));
            let direction = if player == game.you {
                Vec2::new(0.0, 92.0)
            } else {
                match side {
                    SeatSide::Left => Vec2::new(-88.0, 0.0),
                    SeatSide::Top => Vec2::new(0.0, -82.0),
                    SeatSide::Right => Vec2::new(88.0, 0.0),
                }
            };
            add_shengji_failed_throw_card_row(
                commands,
                holder,
                cards,
                shengji_display_trump(game),
                failure.stage,
                direction,
                assets,
            );
        } else {
            add_shengji_card_row(
                commands,
                area,
                cards,
                ShengjiCardSize::Seat,
                shengji_display_trump(game),
                assets,
            );
        }
    }
}

fn add_shengji_own_play(
    commands: &mut Commands,
    table: Entity,
    game: &ShengjiSnapshot,
    assets: &UiAssets,
    previous_trick: Option<&[ShengjiPublicPlay]>,
) {
    let area = spawn_node(
        commands,
        table,
        Node {
            position_type: PositionType::Absolute,
            left: percent(35),
            right: percent(35),
            bottom: px(8),
            min_height: px(104),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            ..default()
        },
        None,
    );
    add_shengji_play_area(
        commands,
        area,
        game,
        game.you,
        SeatSide::Top,
        assets,
        previous_trick,
    );
}

fn add_shengji_collecting_tray(
    commands: &mut Commands,
    table: Entity,
    game: &ShengjiSnapshot,
    cards: &[ShengjiCard],
    score_capture: &ShengjiScoreCaptureEffectState,
    assets: &UiAssets,
) {
    let tray = spawn_node(
        commands,
        table,
        Node {
            position_type: PositionType::Absolute,
            left: px(0),
            top: px(0),
            width: px(410),
            min_height: px(62),
            padding: UiRect::new(px(7), px(78), px(6), px(6)),
            flex_direction: FlexDirection::Column,
            justify_content: JustifyContent::Center,
            border: UiRect::all(px(1)),
            border_radius: BorderRadius::all(px(8)),
            overflow: Overflow::clip(),
            ..default()
        },
        Some(Color::BLACK.with_alpha(0.30)),
    );
    commands
        .entity(tray)
        .insert(BorderColor::all(ACCENT.with_alpha(0.62)));
    let score = spawn_node(
        commands,
        tray,
        Node {
            position_type: PositionType::Absolute,
            right: px(5),
            top: px(4),
            bottom: px(4),
            width: px(68),
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            ..default()
        },
        None,
    );
    add_text(commands, score, "闲家得分", 10.0, MUTED, assets);
    let score_text = add_text(
        commands,
        score,
        displayed_shengji_captured_score(score_capture, shengji_tray_score(game)).to_string(),
        28.0,
        ACCENT,
        assets,
    );
    commands
        .entity(score_text)
        .insert((ShengjiScoreTrayAnchor, ShengjiCollectingScoreText));
    if game.throw_failure.as_ref().is_some_and(|failure| {
        failure.stage == ShengjiThrowFailureStage::Showing && failure.penalty_points > 0
    }) {
        commands
            .entity(score_text)
            .insert(ShengjiThrowPenaltyScorePulse { elapsed: 0.0 });
    }
    if cards.is_empty() {
        add_text(commands, tray, "闲家尚未获得分牌", 12.0, MUTED, assets);
    } else {
        add_shengji_card_row(
            commands,
            tray,
            cards,
            ShengjiCardSize::Score,
            shengji_display_trump(game),
            assets,
        );
    }
}

fn add_shengji_throw_penalty_effect(
    commands: &mut Commands,
    table: Entity,
    game: &ShengjiSnapshot,
    assets: &UiAssets,
) {
    let Some(failure) = game.throw_failure.as_ref().filter(|failure| {
        failure.stage == ShengjiThrowFailureStage::Showing && failure.penalty_points > 0
    }) else {
        return;
    };
    let own_seat = game
        .players
        .iter()
        .find(|player| player.id == game.you)
        .map_or(0, |player| player.seat.0);
    let offender_seat = game
        .players
        .iter()
        .find(|player| player.id == failure.player)
        .map_or(own_seat, |player| player.seat.0);
    let offender_anchor = match (offender_seat + 4 - own_seat) % 4 {
        0 => Vec2::new(50.0, 86.0),
        1 => Vec2::new(20.0, 52.0),
        2 => Vec2::new(50.0, 20.0),
        3 => Vec2::new(80.0, 52.0),
        _ => unreachable!(),
    };
    // 左上得分托盘固定宽 410px，数字位于其最右侧；设计宽度下约为 29%。
    let score_anchor = Vec2::new(29.0, 6.0);
    let dealer_side_penalty = game
        .dealer
        .is_some_and(|dealer| dealer.0 % 2 == failure.player.0 % 2);
    let (source, target, sign) = if dealer_side_penalty {
        (offender_anchor, score_anchor, "+")
    } else {
        (score_anchor, Vec2::new(-4.0, 6.0), "−")
    };
    let effect = add_text(
        commands,
        table,
        format!("{sign}{}", failure.penalty_points),
        24.0,
        Color::NONE,
        assets,
    );
    commands.entity(effect).insert((
        Node {
            position_type: PositionType::Absolute,
            left: percent(source.x),
            top: percent(source.y),
            width: px(60),
            height: px(36),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            ..default()
        },
        ShengjiThrowPenaltyFloat {
            source,
            target,
            elapsed: 0.0,
        },
        UiTransform::IDENTITY,
        TextShadow {
            offset: Vec2::new(1.0, 1.0),
            color: Color::BLACK.with_alpha(0.9),
        },
        GlobalZIndex(840),
        FocusPolicy::Pass,
    ));
}

fn shengji_tray_score(game: &ShengjiSnapshot) -> u32 {
    match &game.phase {
        ShengjiPhaseView::Finished { result, .. } => i64::from(result.trick_points)
            .saturating_add(i64::from(result.penalty_adjustment))
            .max(0) as u32,
        _ => game.collecting_score,
    }
}

fn add_shengji_actions(
    commands: &mut Commands,
    hand_area: Entity,
    game: &ShengjiSnapshot,
    ui: &UiState,
    assets: &UiAssets,
) {
    let actions = spawn_node(
        commands,
        hand_area,
        Node {
            position_type: PositionType::Absolute,
            left: percent(25),
            right: percent(25),
            top: px(0),
            height: px(48),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            column_gap: px(10),
            ..default()
        },
        None,
    );
    match &game.phase {
        ShengjiPhaseView::Burying if game.dealer == Some(game.you) => {
            let count = ui.selected_shengji.len();
            let kitty_size = game.rules.kitty_size();
            if count == kitty_size {
                add_action_button(
                    commands,
                    actions,
                    "埋下底牌",
                    UiAction::SubmitShengjiCards,
                    ButtonKind::Primary,
                    assets,
                );
            } else {
                add_disabled_action_button(
                    commands,
                    actions,
                    &format!("请选择 {kitty_size} 张底牌（{count}/{kitty_size}）"),
                    assets,
                );
            }
        }
        ShengjiPhaseView::Burying => {
            add_text(commands, actions, "等待庄家埋底…", 15.0, MUTED, assets);
        }
        ShengjiPhaseView::BottomCopying { player, .. } if *player == game.you => {
            for (label, target, color) in [
                ("♦", Some(ShengjiSuit::Diamond), DANGER),
                ("♣", Some(ShengjiSuit::Club), TEXT),
                ("♥", Some(ShengjiSuit::Heart), DANGER),
                ("♠", Some(ShengjiSuit::Spade), TEXT),
                ("无主", None, ACCENT),
            ] {
                let cards = shengji_declaration_candidate(game, target);
                add_shengji_bid_button(commands, actions, label, cards, color, true, assets);
            }
            add_action_button(
                commands,
                actions,
                "不抄底",
                UiAction::DeclineBottomCopy,
                ButtonKind::Secondary,
                assets,
            );
        }
        ShengjiPhaseView::BottomCopying { .. } => {
            add_text(commands, actions, "等待抄底…", 15.0, MUTED, assets);
        }
        ShengjiPhaseView::BottomCopyBurying { player } if *player == game.you => {
            let count = ui.selected_shengji.len();
            let kitty_size = game.rules.kitty_size();
            if count == kitty_size {
                add_action_button(
                    commands,
                    actions,
                    "重新埋底",
                    UiAction::SubmitShengjiCards,
                    ButtonKind::Primary,
                    assets,
                );
            } else {
                add_disabled_action_button(
                    commands,
                    actions,
                    &format!("选择 {kitty_size} 张重新埋底（{count}/{kitty_size}）"),
                    assets,
                );
            }
        }
        ShengjiPhaseView::BottomCopyBurying { .. } => {
            add_text(commands, actions, "等待抄底…", 15.0, MUTED, assets);
        }
        ShengjiPhaseView::FiveTrumpCrossing {
            stage: ShengjiFiveTrumpCrossingStage::Deciding,
            eligible,
            decided,
            ..
        } => {
            if eligible.contains(&game.you) && !decided.contains(&game.you) {
                let selected = game
                    .your_hand
                    .iter()
                    .filter(|card| ui.selected_shengji.contains(card))
                    .copied()
                    .collect::<Vec<_>>();
                let includes_all_trumps = game.trump.is_some_and(|trump| {
                    game.your_hand
                        .iter()
                        .filter(|card| trump.is_trump(**card))
                        .all(|card| selected.contains(card))
                });
                if selected.len() == 5 && includes_all_trumps {
                    add_action_button(
                        commands,
                        actions,
                        "五主过江",
                        UiAction::SubmitShengjiCards,
                        ButtonKind::Primary,
                        assets,
                    );
                } else {
                    add_disabled_action_button(
                        commands,
                        actions,
                        &format!("选择全部主牌并补足五张（{}/5）", selected.len()),
                        assets,
                    );
                }
                add_action_button(
                    commands,
                    actions,
                    "不过江",
                    UiAction::DeclineFiveTrumpCrossing,
                    ButtonKind::Secondary,
                    assets,
                );
            } else if decided.contains(&game.you) {
                add_text(
                    commands,
                    actions,
                    "已选择，等待其他玩家…",
                    15.0,
                    MUTED,
                    assets,
                );
            } else {
                add_text(
                    commands,
                    actions,
                    "等待可过江玩家选择…",
                    15.0,
                    MUTED,
                    assets,
                );
            }
        }
        ShengjiPhaseView::FiveTrumpCrossing {
            stage: ShengjiFiveTrumpCrossingStage::Returning,
            crossing,
            returned,
            ..
        } => {
            let partner = PlayerId((game.you.0 + 2) % 4);
            let must_return = crossing.contains(&partner);
            if must_return && !returned.contains(&game.you) {
                let selected_count = game
                    .your_hand
                    .iter()
                    .filter(|card| ui.selected_shengji.contains(card))
                    .count();
                if selected_count == 5 {
                    add_action_button(
                        commands,
                        actions,
                        "归还五张",
                        UiAction::SubmitShengjiCards,
                        ButtonKind::Primary,
                        assets,
                    );
                } else {
                    add_disabled_action_button(
                        commands,
                        actions,
                        &format!("选择五张归还牌（{selected_count}/5）"),
                        assets,
                    );
                }
            } else if returned.contains(&game.you) {
                add_text(
                    commands,
                    actions,
                    "已归还，等待其他玩家…",
                    15.0,
                    MUTED,
                    assets,
                );
            } else {
                add_text(commands, actions, "等待对家完成过江…", 15.0, MUTED, assets);
            }
        }
        ShengjiPhaseView::Playing if game.current_player == Some(game.you) => {
            let required = game
                .trick
                .as_ref()
                .and_then(|trick| trick.plays.first())
                .map(|play| play.play.cards.len());
            let selection_ready = required.map_or_else(
                || !ui.selected_shengji.is_empty(),
                |required| ui.selected_shengji.len() == required,
            );
            if !selection_ready {
                add_disabled_action_button(commands, actions, "出牌", assets);
            } else {
                add_action_button(
                    commands,
                    actions,
                    "出牌",
                    UiAction::SubmitShengjiCards,
                    ButtonKind::Primary,
                    assets,
                );
            }
            add_action_button(
                commands,
                actions,
                "提示",
                UiAction::ShengjiHint,
                ButtonKind::Secondary,
                assets,
            );
        }
        ShengjiPhaseView::Playing => {
            add_text(commands, actions, "等待其他玩家出牌…", 15.0, MUTED, assets);
        }
        ShengjiPhaseView::Finished { .. } => {}
        ShengjiPhaseView::BottomFlipping { .. } => {}
        ShengjiPhaseView::Redealing => {
            add_text(
                commands,
                actions,
                "无人亮主，正在重新发牌…",
                15.0,
                ACCENT,
                assets,
            );
        }
        ShengjiPhaseView::Dealing { .. } | ShengjiPhaseView::BiddingGrace { .. } => {}
    }
}

pub(in crate::app) fn select_forced_shengji_follow_cards(game: &ShengjiSnapshot, ui: &mut UiState) {
    if !matches!(game.phase, ShengjiPhaseView::Playing) || game.current_player != Some(game.you) {
        return;
    }
    let Some(trump) = game.trump else {
        return;
    };
    let Some(lead) = game
        .trick
        .as_ref()
        .and_then(|trick| trick.plays.first())
        .map(|play| &play.play)
    else {
        return;
    };
    ui.selected_shengji
        .extend(shengji_forced_follow_cards(&game.your_hand, lead, trump));
}

fn add_shengji_hand(
    commands: &mut Commands,
    hand_area: Entity,
    game: &ShengjiSnapshot,
    ui: &UiState,
    assets: &UiAssets,
) {
    let hand = spawn_node(
        commands,
        hand_area,
        Node {
            position_type: PositionType::Absolute,
            left: px(0),
            right: px(0),
            bottom: px(0),
            height: px(112),
            flex_direction: FlexDirection::Row,
            flex_wrap: FlexWrap::NoWrap,
            justify_content: JustifyContent::Center,
            align_items: AlignItems::FlexEnd,
            ..default()
        },
        None,
    );
    let mut cards = game.your_hand.clone();
    if let Some(failure) = game
        .throw_failure
        .as_ref()
        .filter(|failure| failure.player == game.you)
    {
        cards.retain(|card| !failure.attempted.contains(card));
    }
    let display_trump = shengji_display_trump(game);
    sort_shengji_cards(&mut cards, shengji_hand_sort_trump(game));
    let hand_len = cards.len();
    let last = cards.len().saturating_sub(1);
    for (index, card) in cards.into_iter().enumerate() {
        add_shengji_hand_card(
            commands,
            hand,
            card,
            index,
            hand_len,
            index == last,
            ui.selected_shengji.contains(&card),
            ui.shengji_card_animations
                .get(&card)
                .copied()
                .unwrap_or_default(),
            display_trump,
            assets,
        );
    }
}

fn add_shengji_hand_card(
    commands: &mut Commands,
    parent: Entity,
    card: ShengjiCard,
    index: usize,
    hand_len: usize,
    is_last: bool,
    selected: bool,
    animation: CardAnimationState,
    trump: Option<ShengjiTrump>,
    assets: &UiAssets,
) {
    let (width, height) = CardSize::Hand.dimensions();
    let image = shengji_card_face(card, assets);
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
            ShengjiHandCardSlot {
                card,
                index,
                hand_len,
                is_last,
                hover_amount: animation.slot_hover_amount,
            },
            UiAction::ToggleShengjiCard,
            RelativeCursorPosition::default(),
            Node {
                width: px(if is_last {
                    width
                } else {
                    shengji_hand_card_reveal(hand_len)
                }),
                height: px(height),
                ..default()
            },
        ))
        .id();
    commands.entity(parent).add_child(button);
    let face = commands
        .spawn((
            ShengjiHandCardVisual {
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
    commands.entity(button).add_child(face);
    add_shengji_trump_stars(commands, face, card, trump, ShengjiCardSize::Hand, assets);
    let overlay = commands
        .spawn((
            ShengjiHandCardSelectionOverlay { index },
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
    commands.entity(face).add_child(overlay);
}

pub(in crate::app) fn queue_shengji_deal_animations(
    client: Option<Res<ClientResource>>,
    mut ui: ResMut<UiState>,
    assets: Res<UiAssets>,
    mut commands: Commands,
) {
    let Some(game) = client
        .as_deref()
        .and_then(|client| client.0.model().shengji_game())
    else {
        ui.observed_shengji_hand.clear();
        ui.shengji_card_animations.clear();
        return;
    };
    if ui.shengji_observed_match != Some(game.match_id)
        || ui.shengji_observed_hand_number != game.hand_number
    {
        ui.shengji_observed_match = Some(game.match_id);
        ui.shengji_observed_hand_number = game.hand_number;
        ui.selected_shengji.clear();
        ui.observed_shengji_hand.clear();
        ui.shengji_card_animations.clear();
    }
    let new_cards = game
        .your_hand
        .iter()
        .copied()
        .filter(|card| !ui.observed_shengji_hand.contains(card))
        .collect::<Vec<_>>();
    let animate_deal = matches!(game.phase, ShengjiPhaseView::Dealing { .. });
    for (index, card) in new_cards.into_iter().enumerate() {
        let delay = if animate_deal {
            index as f32 * SHENGJI_LOCAL_DEAL_INTERVAL
        } else {
            0.0
        };
        ui.shengji_card_animations.insert(
            card,
            CardAnimationState {
                deal_elapsed: if animate_deal { -delay } else { 0.24 },
                dealing: animate_deal,
                ..default()
            },
        );
        if animate_deal && !assets.deal_sounds.is_empty() {
            commands.spawn(PendingDealSound {
                remaining: delay,
                variant: fastrand::usize(..assets.deal_sounds.len()),
            });
        }
    }
    ui.observed_shengji_hand.clone_from(&game.your_hand);
    ui.shengji_card_animations
        .retain(|card, _| game.your_hand.contains(card));
}

pub(in crate::app) fn animate_shengji_hand_cards(
    time: Res<Time>,
    drag: Res<ShengjiCardDragSelection>,
    mut ui: ResMut<UiState>,
    buttons: Query<&Interaction, With<Button>>,
    mut cards: Query<(
        &mut ShengjiHandCardVisual,
        &mut UiTransform,
        &mut Outline,
        &mut BoxShadow,
        &mut ImageNode,
        &mut BorderColor,
    )>,
) {
    let response = 1.0 - (-14.0 * time.delta_secs()).exp();
    let pulse = 0.72 + 0.28 * (time.elapsed_secs() * 7.0).sin();
    for (mut visual, mut transform, mut outline, mut shadow, mut image, mut border) in &mut cards {
        let Ok(interaction) = buttons.get(visual.button) else {
            continue;
        };
        let selected = ui.selected_shengji.contains(&visual.card);
        let hovered = if drag.active {
            visual.index == drag.current
        } else {
            matches!(*interaction, Interaction::Hovered | Interaction::Pressed)
        };
        let hover_target = f32::from(hovered);
        let selected_target = f32::from(selected);
        let transitioning = (hover_target - visual.hover_amount).abs() >= 0.001
            || (selected_target - visual.selected_amount).abs() >= 0.001
            || visual.selected != selected
            || visual.dealing;
        if !transitioning && !hovered && !selected {
            continue;
        }
        visual.selected = selected;
        visual.hover_amount += (hover_target - visual.hover_amount) * response;
        visual.selected_amount += (selected_target - visual.selected_amount) * response;
        if visual.dealing {
            visual.deal_elapsed += time.delta_secs();
            if visual.deal_elapsed >= 0.20 {
                visual.dealing = false;
            }
        }
        let next_animation = CardAnimationState {
            face_hover_amount: visual.hover_amount,
            selected_amount: visual.selected_amount,
            deal_elapsed: visual.deal_elapsed,
            dealing: visual.dealing,
            ..ui.shengji_card_animations
                .get(&visual.card)
                .copied()
                .unwrap_or_default()
        };
        ui.shengji_card_animations
            .insert(visual.card, next_animation);
        let glow = (visual.hover_amount * pulse + visual.selected_amount * 0.72).clamp(0.0, 1.0);
        let pose = hand_card_pose(
            visual.index,
            visual.hand_len,
            visual.hover_amount,
            visual.selected_amount,
            visual.deal_elapsed,
            visual.dealing,
        );
        transform.translation = pose.translation;
        transform.rotation = pose.rotation;
        outline.width = px(0.75 + glow * 1.5);
        outline.color = ACCENT.with_alpha(glow * 0.92);
        image.color = Color::srgb(1.0, 1.0 - glow * 0.035, 1.0 - glow * 0.16);
        border.set_all(if selected { ACCENT } else { BORDER });
        if let Some(style) = shadow.0.first_mut() {
            style.color = ACCENT.with_alpha(glow * 0.58);
            style.spread_radius = px(glow * 2.0);
            style.blur_radius = px(2.0 + glow * 10.0);
        }
    }
}

fn add_shengji_self_panel(
    commands: &mut Commands,
    hand_area: Entity,
    player: &ShengjiPlayerState,
    game: &ShengjiSnapshot,
    assets: &UiAssets,
    avatars: &AvatarImages,
    turn_border_materials: &mut Assets<TurnBorderMaterial>,
    start_transition_active: bool,
) {
    let panel = spawn_node(
        commands,
        hand_area,
        Node {
            position_type: PositionType::Absolute,
            right: px(12),
            bottom: px(12),
            width: px(188),
            height: px(72),
            min_height: px(72),
            padding: UiRect::axes(px(8), px(4)),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            column_gap: px(8),
            border: UiRect::all(px(1)),
            border_radius: BorderRadius::all(px(7)),
            ..default()
        },
        Some(HEADER_BG.with_alpha(0.94)),
    );
    commands.entity(panel).insert(BorderColor::all(BORDER));
    attach_start_game_seat_transition(commands, panel, player.id, start_transition_active);
    decorate_player_panel(commands, panel, assets, 1.0);
    if game.current_player == Some(player.id) {
        add_turn_border_trace(
            commands,
            panel,
            turn_border_materials,
            TurnBorderAnimationKey::new(GameKind::Shengji, game.match_id, player.id),
        );
    }
    let avatar = player.avatar.and_then(|id| avatars.remote.get(&id));
    let avatar_entity = add_avatar(commands, panel, &player.name, avatar, 34.0, assets);
    commands
        .entity(avatar_entity)
        .insert(PlayerAvatarAnchor(player.id));
    let info = spawn_node(
        commands,
        panel,
        Node {
            flex_direction: FlexDirection::Column,
            ..default()
        },
        None,
    );
    add_text(commands, info, &player.name, 14.0, TEXT, assets);
    let level = add_text(
        commands,
        info,
        shengji_level_label(game.levels[usize::from(player.id.0 % 2)]),
        11.0,
        ACCENT,
        assets,
    );
    commands
        .entity(level)
        .insert(ShengjiLevelIndicator { base_color: ACCENT });
    if game.dealer == Some(player.id) {
        let dealer = spawn_node(
            commands,
            panel,
            Node {
                position_type: PositionType::Absolute,
                right: px(5),
                top: px(4),
                width: px(24),
                height: px(24),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                border_radius: BorderRadius::all(percent(50)),
                ..default()
            },
            Some(ACCENT),
        );
        commands.entity(dealer).insert(ShengjiDealerBadge);
        add_text(commands, dealer, "庄", 12.0, Color::BLACK, assets);
    }
}

fn add_shengji_result(
    commands: &mut Commands,
    table: Entity,
    game: &ShengjiSnapshot,
    result: &leocard_protocol::ShengjiHandResultView,
    buried: &[ShengjiCard],
    assets: &UiAssets,
    avatars: &AvatarImages,
    animation: &ShengjiSettlementAnimation,
) {
    let kitty_award = u32::from(result.kitty_points).saturating_mul(result.kitty_multiplier);
    let stage = spawn_node(
        commands,
        table,
        Node {
            position_type: PositionType::Absolute,
            // 居中且保持足够紧凑，左边缘不会压住左上角的闲家得分框。
            left: percent(30),
            right: percent(30),
            top: percent(3),
            height: percent(32),
            min_height: px(205),
            padding: UiRect::all(px(20)),
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            row_gap: px(5),
            border_radius: BorderRadius::all(px(12)),
            ..default()
        },
        Some(HEADER_BG.with_alpha(0.94)),
    );
    commands.entity(stage).insert((
        BorderColor::all(Color::NONE),
        BoxShadow::new(Color::BLACK.with_alpha(0.48), px(2), px(7), px(0), px(12)),
        GlobalZIndex(1120),
        FocusPolicy::Pass,
    ));
    decorate_panel_skin(commands, stage, PanelSkin::Section, assets);

    let kitty_title = add_text(commands, stage, "底牌", 17.0, ACCENT, assets);
    commands
        .entity(kitty_title)
        .insert(ShengjiTimedReveal { delay: 0.0 });
    let card_row = spawn_node(
        commands,
        stage,
        Node {
            height: px(70),
            min_width: px(390),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            column_gap: px(5),
            ..default()
        },
        None,
    );
    let trump = shengji_display_trump(game);
    for (index, card) in buried.iter().copied().enumerate() {
        let entity = commands
            .spawn((
                Node {
                    width: px(45),
                    height: px(62),
                    border_radius: BorderRadius::all(px(4)),
                    ..default()
                },
                ImageNode::new(shengji_card_face(card, assets)).with_color(Color::NONE),
                UiTransform {
                    translation: Val2::px(0.0, 16.0),
                    scale: Vec2::splat(0.78),
                    ..default()
                },
                Visibility::Hidden,
                ShengjiKittyRevealCard { index },
            ))
            .id();
        commands.entity(card_row).add_child(entity);
        add_shengji_trump_stars(
            commands,
            entity,
            card,
            trump,
            ShengjiCardSize::Score,
            assets,
        );
    }

    let score_summary = spawn_node(
        commands,
        stage,
        Node {
            width: percent(100),
            height: px(44),
            padding: UiRect::horizontal(px(16)),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::SpaceBetween,
            column_gap: px(20),
            ..default()
        },
        None,
    );
    let kitty_score_line = spawn_node(
        commands,
        score_summary,
        Node {
            height: px(40),
            flex_grow: 1.0,
            align_items: AlignItems::Center,
            justify_content: JustifyContent::FlexStart,
            column_gap: px(8),
            ..default()
        },
        None,
    );
    commands.entity(kitty_score_line).insert((
        ShengjiTimedReveal {
            delay: SHENGJI_KITTY_SCORE_DELAY,
        },
        Visibility::Hidden,
    ));
    add_text(commands, kitty_score_line, "底牌分数", 14.0, MUTED, assets);
    let kitty_score = add_text(
        commands,
        kitty_score_line,
        result.kitty_points.to_string(),
        28.0,
        ACCENT,
        assets,
    );
    commands.entity(kitty_score).insert((
        ShengjiKittyScoreAnchor,
        ShengjiKittyScoreText {
            base: u32::from(result.kitty_points),
            awarded: kitty_award,
        },
        UiTransform::IDENTITY,
    ));
    if result.kitty_multiplier > 0 {
        let multiplier = add_text(
            commands,
            kitty_score_line,
            format!("×{}", result.kitty_multiplier),
            25.0,
            READY,
            assets,
        );
        commands.entity(multiplier).insert((
            ShengjiKittyMultiplier,
            UiTransform {
                translation: Val2::px(42.0, -24.0),
                scale: Vec2::splat(0.82),
                ..default()
            },
            Visibility::Hidden,
        ));
    } else {
        add_text(commands, kitty_score_line, "庄家守底", 13.0, MUTED, assets);
    }

    let total_box = spawn_node(
        commands,
        score_summary,
        Node {
            min_width: px(170),
            height: px(40),
            flex_grow: 1.0,
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::Center,
            justify_content: JustifyContent::FlexEnd,
            column_gap: px(8),
            ..default()
        },
        None,
    );
    commands.entity(total_box).insert((
        ShengjiTimedReveal {
            delay: SHENGJI_TOTAL_LABEL_DELAY,
        },
        Visibility::Hidden,
    ));
    add_text(commands, total_box, "闲家总得分", 13.0, MUTED, assets);
    let total_text = add_text(commands, total_box, "0", 28.0, ACCENT, assets);
    commands.entity(total_text).insert((
        ShengjiSettlementTotalAnchor,
        ShengjiSettlementTotalText {
            target: result.collecting_score,
        },
        UiTransform::IDENTITY,
    ));

    add_shengji_settlement_modal(commands, table, game, result, assets, avatars, animation);
}

fn add_shengji_settlement_modal(
    commands: &mut Commands,
    table: Entity,
    game: &ShengjiSnapshot,
    result: &leocard_protocol::ShengjiHandResultView,
    assets: &UiAssets,
    avatars: &AvatarImages,
    animation: &ShengjiSettlementAnimation,
) {
    let modal = spawn_node(
        commands,
        table,
        Node {
            position_type: PositionType::Absolute,
            left: percent(25),
            right: percent(25),
            top: percent(37),
            height: percent(50),
            min_height: px(300),
            padding: UiRect::all(px(24)),
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            row_gap: px(5),
            border_radius: BorderRadius::all(px(12)),
            ..default()
        },
        Some(Color::NONE),
    );
    commands.entity(modal).insert((
        ShengjiSettlementModal,
        UiTransform {
            translation: Val2::px(0.0, 72.0),
            ..default()
        },
        GlobalZIndex(1220),
        FocusPolicy::Block,
        Visibility::Hidden,
    ));
    let texture = decorate_panel_skin(commands, modal, PanelSkin::Window, assets);
    commands
        .entity(texture)
        .insert(ShengjiSettlementPanelTexture);
    add_text(commands, modal, "本局结算", 24.0, ACCENT, assets);
    let outcome = add_text(
        commands,
        modal,
        shengji_settlement_outcome(result, game.rules.deck_count),
        20.0,
        TEXT,
        assets,
    );
    commands
        .entity(outcome)
        .insert(ShengjiSettlementOutcomeText);

    let list = spawn_node(
        commands,
        modal,
        Node {
            width: percent(100),
            flex_grow: 1.0,
            flex_direction: FlexDirection::Column,
            row_gap: px(4),
            ..default()
        },
        None,
    );
    let mut players = game.players.iter().collect::<Vec<_>>();
    players.sort_by_key(|player| player.seat.0);
    for (index, player) in players.iter().enumerate() {
        let delay =
            SHENGJI_SETTLEMENT_MODAL_DELAY + 0.46 + index as f32 * SHENGJI_SETTLEMENT_ROW_INTERVAL;
        let row = spawn_node(
            commands,
            list,
            Node {
                width: percent(100),
                min_height: px(38),
                padding: UiRect::axes(px(9), px(4)),
                align_items: AlignItems::Center,
                column_gap: px(9),
                border_radius: BorderRadius::all(px(7)),
                ..default()
            },
            Some(Color::NONE),
        );
        commands.entity(row).insert((
            ShengjiSettlementRow { delay },
            UiTransform {
                translation: Val2::px(0.0, 16.0),
                ..default()
            },
            Visibility::Hidden,
        ));
        let avatar = player.avatar.and_then(|id| avatars.remote.get(&id));
        add_ready_avatar(
            commands,
            row,
            &player.name,
            avatar,
            28.0,
            player.ready,
            assets,
        );
        let name = spawn_node(
            commands,
            row,
            Node {
                flex_grow: 1.0,
                min_width: px(0),
                flex_direction: FlexDirection::Column,
                ..default()
            },
            None,
        );
        add_text(commands, name, &player.name, 15.0, TEXT, assets);
        add_text(
            commands,
            name,
            format!("平台积分 {}", player.reference_points),
            10.0,
            MUTED,
            assets,
        );
        let delta = result
            .reference_changes
            .iter()
            .find(|change| change.player == player.id)
            .map_or(0, |change| change.delta);
        add_text(
            commands,
            row,
            format!("{delta:+}"),
            20.0,
            if delta >= 0 { READY } else { DANGER },
            assets,
        );
    }

    let actions_delay = SHENGJI_SETTLEMENT_MODAL_DELAY
        + 0.46
        + players.len() as f32 * SHENGJI_SETTLEMENT_ROW_INTERVAL
        + 0.26;
    let actions = spawn_node(
        commands,
        modal,
        Node {
            width: percent(100),
            min_height: px(47),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            column_gap: px(8),
            ..default()
        },
        None,
    );
    commands.entity(actions).insert((
        ShengjiSettlementActions {
            delay: actions_delay,
        },
        Visibility::Hidden,
    ));
    let ready = game
        .players
        .iter()
        .find(|player| player.id == game.you)
        .is_some_and(|player| player.ready);
    if ready {
        add_disabled_action_button(commands, actions, "已准备", assets);
    } else {
        add_action_button(
            commands,
            actions,
            "准备下一局",
            UiAction::PlayAgain,
            ButtonKind::Primary,
            assets,
        );
    }
    if game.you == game.host {
        add_action_button(
            commands,
            actions,
            "结束并返回大厅",
            UiAction::ReturnToLobby,
            ButtonKind::Secondary,
            assets,
        );
    }
    let _ = animation;
}

pub(in crate::app) fn shengji_settlement_outcome(
    result: &leocard_protocol::ShengjiHandResultView,
    deck_count: u8,
) -> String {
    shengji_settlement_outcome_for_score(result.collecting_score, result.promoted_steps, deck_count)
}

pub(in crate::app) fn shengji_settlement_outcome_for_score(
    collecting_score: u32,
    promoted_steps: u8,
    deck_count: u8,
) -> String {
    let (small_light, takeover) = match deck_count {
        4 => (80, 160),
        3 => (60, 120),
        _ => (40, 80),
    };
    if collecting_score == 0 {
        "闲家大光".to_owned()
    } else if collecting_score < small_light {
        "闲家小光".to_owned()
    } else if collecting_score < takeover {
        "闲家脱贫".to_owned()
    } else if promoted_steps == 0 {
        "闲家上台".to_owned()
    } else {
        format!("闲家升{promoted_steps}级")
    }
}

#[derive(Clone, Copy)]
enum ShengjiCardSize {
    Hand,
    Seat,
    Score,
}

#[derive(Clone, Copy)]
struct ShengjiFailedThrowCardSpec {
    stage: ShengjiThrowFailureStage,
    direction: Vec2,
}

fn add_shengji_card_row(
    commands: &mut Commands,
    parent: Entity,
    cards: &[ShengjiCard],
    size: ShengjiCardSize,
    trump: Option<ShengjiTrump>,
    assets: &UiAssets,
) -> Entity {
    add_shengji_card_row_internal(commands, parent, cards, size, trump, None, assets)
}

fn add_shengji_failed_throw_card_row(
    commands: &mut Commands,
    parent: Entity,
    cards: &[ShengjiCard],
    trump: Option<ShengjiTrump>,
    stage: ShengjiThrowFailureStage,
    direction: Vec2,
    assets: &UiAssets,
) -> Entity {
    add_shengji_card_row_internal(
        commands,
        parent,
        cards,
        ShengjiCardSize::Seat,
        trump,
        Some(ShengjiFailedThrowCardSpec { stage, direction }),
        assets,
    )
}

fn add_shengji_card_row_internal(
    commands: &mut Commands,
    parent: Entity,
    cards: &[ShengjiCard],
    size: ShengjiCardSize,
    trump: Option<ShengjiTrump>,
    failed_throw: Option<ShengjiFailedThrowCardSpec>,
    assets: &UiAssets,
) -> Entity {
    let (width, height, mut reveal): (f32, f32, f32) = match size {
        ShengjiCardSize::Hand => {
            let (width, height) = CardSize::Hand.dimensions();
            (width, height, HAND_CARD_REVEAL)
        }
        ShengjiCardSize::Seat => (58.0, 79.0, 24.0),
        ShengjiCardSize::Score => (36.0, 49.0, 14.0),
    };
    if matches!(size, ShengjiCardSize::Score) && cards.len() > 1 {
        // 将两至四副牌的全部分牌压进左上角托盘，而不是裁掉尾部牌。
        reveal = reveal.min(280.0 / cards.len().saturating_sub(1) as f32);
    }
    let mut cards = cards.to_vec();
    sort_shengji_cards(&mut cards, trump);
    let last = cards.len().saturating_sub(1);
    let row = spawn_node(
        commands,
        parent,
        Node {
            height: px(height),
            flex_direction: FlexDirection::Row,
            flex_wrap: FlexWrap::NoWrap,
            align_items: AlignItems::Center,
            ..default()
        },
        None,
    );
    for (index, card) in cards.into_iter().enumerate() {
        let entity = commands
            .spawn((
                Node {
                    width: px(width),
                    height: px(height),
                    margin: UiRect::right(px(if index == last { 0.0 } else { reveal - width })),
                    border_radius: BorderRadius::all(px(4)),
                    ..default()
                },
                ImageNode::new(shengji_card_face(card, assets)),
                ZIndex(index as i32),
            ))
            .id();
        commands.entity(row).add_child(entity);
        if let Some(failed_throw) = failed_throw {
            commands.entity(entity).insert((
                ShengjiFailedThrowCard {
                    index,
                    count: last + 1,
                    stage: failed_throw.stage,
                    direction: failed_throw.direction,
                    elapsed: 0.0,
                },
                UiTransform::IDENTITY,
                Visibility::Visible,
            ));
        }
        add_shengji_trump_stars(commands, entity, card, trump, size, assets);
    }
    row
}

fn add_shengji_trump_stars(
    commands: &mut Commands,
    card_entity: Entity,
    card: ShengjiCard,
    trump: Option<ShengjiTrump>,
    size: ShengjiCardSize,
    assets: &UiAssets,
) {
    let count = shengji_trump_star_count(card, trump);
    if count == 0 {
        return;
    }
    let (left, bottom, font_size) = match size {
        ShengjiCardSize::Hand => (4.0, 4.0, 14.0),
        ShengjiCardSize::Seat => (3.0, 3.0, 12.0),
        ShengjiCardSize::Score => (2.0, 2.0, 8.0),
    };
    let marker = add_text(
        commands,
        card_entity,
        if count == 2 { "★\n★" } else { "★" }.to_owned(),
        font_size,
        Color::srgb(1.0, 0.76, 0.08),
        assets,
    );
    commands.entity(marker).insert((
        Node {
            position_type: PositionType::Absolute,
            left: px(left),
            bottom: px(bottom),
            ..default()
        },
        TextShadow {
            offset: Vec2::new(1.0, 1.0),
            color: Color::BLACK.with_alpha(0.92),
        },
        ZIndex(5),
        FocusPolicy::Pass,
    ));
}

pub(in crate::app) fn shengji_trump_star_count(
    card: ShengjiCard,
    trump: Option<ShengjiTrump>,
) -> u8 {
    let Some(trump) = trump else {
        return 0;
    };
    if !trump.is_trump(card) {
        return 0;
    }
    if matches!(card.rank(), ShengjiRank::SmallJoker | ShengjiRank::BigJoker)
        || (card.rank() == trump.level && card.suit() == trump.suit && trump.suit.is_some())
    {
        2
    } else {
        1
    }
}

pub(in crate::app) fn sort_shengji_cards(cards: &mut [ShengjiCard], trump: Option<ShengjiTrump>) {
    cards.sort_by(|left, right| {
        shengji_display_key(*right, trump)
            .cmp(&shengji_display_key(*left, trump))
            .then_with(|| right.deck().cmp(&left.deck()))
    });
}

fn shengji_display_key(card: ShengjiCard, trump: Option<ShengjiTrump>) -> (u8, u8, u8, u8) {
    let is_trump = trump.is_some_and(|trump| trump.is_trump(card));
    let strength = trump.map_or_else(
        || shengji_rank_order(card.rank()),
        |trump| {
            if is_trump {
                trump.strength(card)
            } else {
                shengji_rank_order(card.rank())
            }
        },
    );
    let category = if is_trump {
        4
    } else {
        card.suit().map_or(4, ShengjiSuit::bid_strength)
    };
    // 所有主牌共用同一个 category；副级牌的 strength 也完全相同，因此必须
    // 在牌副编号之前再按黑、红、梅、方分组，否则两副实体牌会交错成乱序。
    let suit_order = card.suit().map_or(4, ShengjiSuit::bid_strength);
    (u8::from(is_trump), category, strength, suit_order)
}

pub(in crate::app) fn shengji_card_face(card: ShengjiCard, assets: &UiAssets) -> Handle<Image> {
    let rank = match card.rank() {
        ShengjiRank::Two => Rank::Two,
        ShengjiRank::Three => Rank::Three,
        ShengjiRank::Four => Rank::Four,
        ShengjiRank::Five => Rank::Five,
        ShengjiRank::Six => Rank::Six,
        ShengjiRank::Seven => Rank::Seven,
        ShengjiRank::Eight => Rank::Eight,
        ShengjiRank::Nine => Rank::Nine,
        ShengjiRank::Ten => Rank::Ten,
        ShengjiRank::Jack => Rank::Jack,
        ShengjiRank::Queen => Rank::Queen,
        ShengjiRank::King => Rank::King,
        ShengjiRank::Ace => Rank::Ace,
        ShengjiRank::SmallJoker | ShengjiRank::BigJoker => Rank::Joker,
    };
    let suit = match (card.suit(), card.rank()) {
        (Some(ShengjiSuit::Diamond), _) => Suit::Diamond,
        (Some(ShengjiSuit::Club), _) => Suit::Club,
        (Some(ShengjiSuit::Heart), _) => Suit::Heart,
        (Some(ShengjiSuit::Spade), _) => Suit::Spade,
        (None, ShengjiRank::SmallJoker) => Suit::Club,
        (None, ShengjiRank::BigJoker) => Suit::Spade,
        _ => unreachable!("合法双升牌的花色与点数组合"),
    };
    assets
        .cards
        .get(&(rank, suit))
        .expect("双升牌面已经加载")
        .clone()
}

fn shengji_rank_order(rank: ShengjiRank) -> u8 {
    match rank {
        ShengjiRank::Two => 0,
        ShengjiRank::Three => 1,
        ShengjiRank::Four => 2,
        ShengjiRank::Five => 3,
        ShengjiRank::Six => 4,
        ShengjiRank::Seven => 5,
        ShengjiRank::Eight => 6,
        ShengjiRank::Nine => 7,
        ShengjiRank::Ten => 8,
        ShengjiRank::Jack => 9,
        ShengjiRank::Queen => 10,
        ShengjiRank::King => 11,
        ShengjiRank::Ace => 12,
        ShengjiRank::SmallJoker => 13,
        ShengjiRank::BigJoker => 14,
    }
}

pub(in crate::app) fn shengji_current_level(game: &ShengjiSnapshot) -> ShengjiRank {
    game.trump.map_or(game.bidding_level, |trump| trump.level)
}

pub(in crate::app) fn shengji_display_trump(game: &ShengjiSnapshot) -> Option<ShengjiTrump> {
    game.trump.or_else(|| {
        game.declaration.as_ref().map(|declaration| {
            ShengjiTrump::new(shengji_current_level(game), declaration.trump.trump_suit())
                .expect("双升级牌始终是普通点数")
                .with_constant_trump(game.rules.constant_trump)
        })
    })
}

/// 手牌排序在尚无人亮主时也使用一个仅用于排序的临时无主规则，使本局
/// 级牌紧跟在大小王右侧。渲染星标仍使用 `shengji_display_trump`，不会把
/// 尚未正式确定的主牌状态提前公开。
pub(in crate::app) fn shengji_hand_sort_trump(game: &ShengjiSnapshot) -> Option<ShengjiTrump> {
    shengji_display_trump(game).or_else(|| {
        ShengjiTrump::new(game.bidding_level, None)
            .map(|trump| trump.with_constant_trump(game.rules.constant_trump))
            .ok()
    })
}

fn shengji_level_label(rank: ShengjiRank) -> String {
    format!("打 {}", shengji_rank_label(rank))
}

fn shengji_rank_label(rank: ShengjiRank) -> &'static str {
    match rank {
        ShengjiRank::Two => "2",
        ShengjiRank::Three => "3",
        ShengjiRank::Four => "4",
        ShengjiRank::Five => "5",
        ShengjiRank::Six => "6",
        ShengjiRank::Seven => "7",
        ShengjiRank::Eight => "8",
        ShengjiRank::Nine => "9",
        ShengjiRank::Ten => "10",
        ShengjiRank::Jack => "J",
        ShengjiRank::Queen => "Q",
        ShengjiRank::King => "K",
        ShengjiRank::Ace => "A",
        ShengjiRank::SmallJoker => "小王",
        ShengjiRank::BigJoker => "大王",
    }
}
