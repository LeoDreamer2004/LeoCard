use super::*;

const MAHJONG_TILE_SHADER: &str = "shaders/mahjong_tile.wgsl";
const MAHJONG_DEAL_MOVE_DURATION: f32 = 0.28;
const MAHJONG_CLAIM_FLIGHT_DELAY: f32 = 0.18;
const MAHJONG_CLAIM_FLIGHT_DURATION: f32 = 0.52;
const MAHJONG_CLAIM_HAND_SHIFT_DURATION: f32 = 0.34;
const MAHJONG_CLAIM_PRESENTATION_DURATION: f32 = 1.42;
const MAHJONG_FLOWER_PRESENTATION_DURATION: f32 = 1.18;
const MAHJONG_WIN_EFFECT_DURATION: f32 = 0.92;
const MAHJONG_OWN_HAND_LEFT: f32 = 315.0;
const MAHJONG_OWN_MELD_WIDTH: f32 = 140.0;
const MAHJONG_REMOTE_MELD_WIDTH: f32 = 78.0;

const fn mahjong_claim_landing_time() -> f32 {
    MAHJONG_CLAIM_FLIGHT_DELAY + MAHJONG_CLAIM_FLIGHT_DURATION
}

#[derive(AsBindGroup, Asset, TypePath, Debug, Clone)]
pub(super) struct MahjongTileMaterial {
    /// x: 交互；y: 正背面；z: 可见度；w: -4 自家副露、-3 对家、-2 侧家、-1 自家、2 双层墙、3 下层牌。
    #[uniform(0)]
    params: Vec4,
    /// 屏幕左上方光源转换到牌的局部坐标后的方向。
    #[uniform(0)]
    lighting: Vec4,
    #[texture(1)]
    #[sampler(2)]
    glyph: Handle<Image>,
    #[texture(3)]
    #[sampler(4)]
    height: Handle<Image>,
}

impl UiMaterial for MahjongTileMaterial {
    fn fragment_shader() -> ShaderRef {
        MAHJONG_TILE_SHADER.into()
    }
}

fn mahjong_local_light(orientation: u8) -> Vec4 {
    let direction = match orientation {
        0 => Vec2::new(-0.50, -0.72),
        1 => Vec2::new(0.72, -0.50),
        2 => Vec2::new(0.50, 0.72),
        _ => Vec2::new(-0.72, 0.50),
    };
    direction.extend(0.0).extend(0.0)
}

fn mahjong_local_shadow(orientation: u8) -> Vec2 {
    match orientation {
        0 => Vec2::new(2.0, 5.0),
        1 => Vec2::new(-5.0, 2.0),
        2 => Vec2::new(-2.0, -5.0),
        _ => Vec2::new(5.0, -2.0),
    }
}

#[derive(Component)]
pub(in crate::app) struct MahjongHandTile {
    lift: f32,
    base_rotation: f32,
    index: i32,
}

#[derive(Component)]
pub(in crate::app) struct MahjongDealTile {
    elapsed: f32,
    start_offset: Vec2,
    start_rotation: f32,
    final_offset: Vec2,
    final_rotation: f32,
    final_shadow_alpha: f32,
}

#[derive(Component)]
pub(in crate::app) struct MahjongTurnArrow {
    slot: f32,
}

#[derive(Component)]
pub(in crate::app) struct MahjongWinningHand {
    relative: u8,
    base_rotation: f32,
}

#[derive(Clone, Debug)]
struct ActiveMahjongClaimPresentation {
    match_id: MatchId,
    player: PlayerId,
    source: Option<PlayerId>,
    tile: Option<MahjongTile>,
    claim: MahjongClaim,
    shift_hand: bool,
    elapsed: f32,
}

#[derive(Clone, Debug)]
struct ActiveMahjongFlowerPresentation {
    match_id: MatchId,
    player: PlayerId,
    elapsed: f32,
}

#[derive(Resource, Default)]
pub(in crate::app) struct MahjongClaimPresentationState {
    active: Option<ActiveMahjongClaimPresentation>,
    queued: VecDeque<ActiveMahjongClaimPresentation>,
    flowers: Vec<ActiveMahjongFlowerPresentation>,
    observed_match: Option<MatchId>,
}

#[derive(Component)]
pub(in crate::app) struct MahjongClaimFlight {
    player: PlayerId,
    tile: MahjongTile,
    start: Vec2,
    control: Vec2,
    target: Vec2,
    start_angle: f32,
    target_angle: f32,
    target_scale: f32,
}

#[derive(Component)]
pub(in crate::app) struct MahjongClaimLabel {
    player: PlayerId,
    text: Entity,
}

#[derive(Component)]
pub(in crate::app) struct MahjongFlowerLabel {
    player: PlayerId,
    text: Entity,
}

#[derive(Component)]
pub(in crate::app) struct MahjongClaimHandShift {
    player: PlayerId,
    distance: f32,
}

#[derive(Component)]
pub(in crate::app) struct MahjongClaimHeldTile {
    player: PlayerId,
}

#[derive(Component)]
pub(in crate::app) struct MahjongWinEffect {
    text: Entity,
}

#[derive(Clone, Copy)]
struct MahjongDealSpec {
    start_offset: Vec2,
    start_rotation: f32,
}

pub(in crate::app) struct MahjongTableVisuals<'a> {
    pub(in crate::app) assets: &'a UiAssets,
    pub(in crate::app) avatars: &'a AvatarImages,
    pub(in crate::app) appearance: &'a TableAppearance,
    pub(in crate::app) brightness: f32,
    pub(in crate::app) vignette: f32,
    pub(in crate::app) table_materials: &'a mut Assets<TableBackgroundMaterial>,
    pub(in crate::app) tile_materials: &'a mut Assets<MahjongTileMaterial>,
    pub(in crate::app) game_summary: &'a GameSummaryAnimation,
    pub(in crate::app) claim_presentation: &'a MahjongClaimPresentationState,
}

pub(in crate::app) fn sync_mahjong_claim_presentation(
    mut client: Option<ResMut<ClientResource>>,
    mut presentation: ResMut<MahjongClaimPresentationState>,
) {
    let Some(client) = client.as_deref_mut() else {
        *presentation = MahjongClaimPresentationState::default();
        return;
    };
    let events = client.0.take_mahjong_events();
    let Some(match_id) = client.0.model().mahjong_game().map(|game| game.match_id) else {
        *presentation = MahjongClaimPresentationState::default();
        return;
    };
    if presentation.observed_match != Some(match_id) {
        *presentation = MahjongClaimPresentationState {
            observed_match: Some(match_id),
            ..default()
        };
    }
    for event in events {
        if let MahjongEvent::FlowerReplaced { player } = &event {
            if let Some(active) = presentation
                .flowers
                .iter_mut()
                .find(|active| active.player == *player)
            {
                active.elapsed = 0.0;
            } else {
                presentation.flowers.push(ActiveMahjongFlowerPresentation {
                    match_id,
                    player: *player,
                    elapsed: 0.0,
                });
            }
            continue;
        }
        let active = match event {
            MahjongEvent::ClaimResolved {
                player,
                source,
                tile,
                claim,
            } if matches!(
                claim,
                MahjongClaim::Chow { .. } | MahjongClaim::Pung | MahjongClaim::Kong
            ) =>
            {
                ActiveMahjongClaimPresentation {
                    match_id,
                    player,
                    source: Some(source),
                    tile: Some(tile),
                    claim,
                    shift_hand: true,
                    elapsed: 0.0,
                }
            }
            MahjongEvent::KongDeclared { player, added, .. } => ActiveMahjongClaimPresentation {
                match_id,
                player,
                source: None,
                tile: None,
                claim: MahjongClaim::Kong,
                shift_hand: !added,
                elapsed: 0.0,
            },
            _ => continue,
        };
        if presentation.active.is_none() {
            presentation.active = Some(active);
        } else {
            presentation.queued.push_back(active);
        }
    }
}

pub(in crate::app) fn advance_mahjong_claim_presentation(
    time: Res<Time>,
    mut presentation: ResMut<MahjongClaimPresentationState>,
    mut ui: ResMut<UiState>,
) {
    for flower in &mut presentation.flowers {
        flower.elapsed += time.delta_secs();
    }
    let flower_count = presentation.flowers.len();
    presentation
        .flowers
        .retain(|flower| flower.elapsed < MAHJONG_FLOWER_PRESENTATION_DURATION);
    if presentation.flowers.len() != flower_count {
        ui.dirty = true;
    }
    let Some(active) = presentation.active.as_mut() else {
        return;
    };
    let previous = active.elapsed;
    active.elapsed += time.delta_secs();
    if active.source.is_some()
        && previous < mahjong_claim_landing_time()
        && active.elapsed >= mahjong_claim_landing_time()
    {
        ui.dirty = true;
    }
    if active.elapsed >= MAHJONG_CLAIM_PRESENTATION_DURATION {
        presentation.active = presentation.queued.pop_front();
        ui.dirty = true;
    }
}

pub(in crate::app) fn mahjong_tile_asset_path(kind: MahjongTileKind) -> String {
    let name = match kind {
        MahjongTileKind::Suited {
            suit: MahjongSuit::Characters,
            rank,
        } => format!("characters-{rank}"),
        MahjongTileKind::Suited {
            suit: MahjongSuit::Bamboo,
            rank,
        } => format!("bamboo-{rank}"),
        MahjongTileKind::Suited {
            suit: MahjongSuit::Dots,
            rank,
        } => format!("dots-{rank}"),
        MahjongTileKind::Wind(MahjongWind::East) => "wind-east".to_owned(),
        MahjongTileKind::Wind(MahjongWind::South) => "wind-south".to_owned(),
        MahjongTileKind::Wind(MahjongWind::West) => "wind-west".to_owned(),
        MahjongTileKind::Wind(MahjongWind::North) => "wind-north".to_owned(),
        MahjongTileKind::Dragon(MahjongDragon::Red) => "dragon-red".to_owned(),
        MahjongTileKind::Dragon(MahjongDragon::Green) => "dragon-green".to_owned(),
        MahjongTileKind::Dragon(MahjongDragon::White) => "dragon-white".to_owned(),
        MahjongTileKind::Flower(MahjongFlower::Spring) => "season-spring".to_owned(),
        MahjongTileKind::Flower(MahjongFlower::Summer) => "season-summer".to_owned(),
        MahjongTileKind::Flower(MahjongFlower::Autumn) => "season-autumn".to_owned(),
        MahjongTileKind::Flower(MahjongFlower::Winter) => "season-winter".to_owned(),
        MahjongTileKind::Flower(MahjongFlower::Plum) => "flower-plum".to_owned(),
        MahjongTileKind::Flower(MahjongFlower::Orchid) => "flower-orchid".to_owned(),
        MahjongTileKind::Flower(MahjongFlower::Bamboo) => "flower-bamboo".to_owned(),
        MahjongTileKind::Flower(MahjongFlower::Chrysanthemum) => "flower-chrysanthemum".to_owned(),
    };
    format!("cards/mahjong/hong-kong/{name}.png")
}

pub(in crate::app) fn mahjong_tile_height_asset_path(kind: MahjongTileKind) -> String {
    mahjong_tile_asset_path(kind).replace("/hong-kong/", "/hong-kong-height/")
}

pub(in crate::app) fn render_mahjong_lobby(
    commands: &mut Commands,
    root: Entity,
    client: &ClientResource,
    lobby: &leocard_protocol::LobbySnapshot,
    assets: &UiAssets,
    avatars: &AvatarImages,
) {
    let rules = *lobby.mahjong_rules().expect("麻将大厅应携带对应规则");
    let connected = connected_lobby_player_count(lobby);
    let content = spawn_node(
        commands,
        root,
        Node {
            width: percent(100),
            max_width: px(1180),
            flex_grow: 1.0,
            align_self: AlignSelf::Center,
            padding: UiRect::all(px(22)),
            flex_direction: FlexDirection::Row,
            column_gap: px(18),
            align_items: AlignItems::Stretch,
            ..default()
        },
        None,
    );
    let rules_panel = add_panel(
        commands,
        content,
        Node {
            min_width: px(340),
            flex_basis: px(390),
            flex_grow: 1.0,
            flex_direction: FlexDirection::Column,
            row_gap: px(15),
            ..default()
        },
        PANEL,
        PanelSkin::Section,
        assets,
    );
    add_section_title(commands, rules_panel, "国标麻将配置", assets);
    add_text(
        commands,
        rules_panel,
        format!("当前人数 {connected}/4，需要四人开局。采用 2014 版国标规则。"),
        13.0,
        MUTED,
        assets,
    );
    let can_configure = client.0.model().you() == lobby.host;
    let lengths = [
        MahjongMatchLength::SingleHand,
        MahjongMatchLength::EastRound,
        MahjongMatchLength::HalfGame,
        MahjongMatchLength::FullGame,
    ];
    let index = lengths
        .iter()
        .position(|length| *length == rules.match_length)
        .unwrap_or_default();
    add_mahjong_rule_config_row(
        commands,
        rules_panel,
        MahjongRuleConfigRow {
            label: "场次",
            value: match rules.match_length {
                MahjongMatchLength::SingleHand => "单局结算",
                MahjongMatchLength::EastRound => "东风场（4 局）",
                MahjongMatchLength::HalfGame => "半庄场（8 局）",
                MahjongMatchLength::FullGame => "全庄场（16 局）",
            }
            .to_owned(),
            help: "每盘结算后全员可直接准备下一盘，不会返回大厅。单局模式仍会继续轮庄。",
            editable: can_configure,
            previous: can_configure.then_some(MahjongRuleSet {
                match_length: lengths[(index + lengths.len() - 1) % lengths.len()],
                ..rules
            }),
            next: can_configure.then_some(MahjongRuleSet {
                match_length: lengths[(index + 1) % lengths.len()],
                ..rules
            }),
        },
        assets,
    );
    let minimum_toggled = MahjongRuleSet {
        minimum_eight_points: !rules.minimum_eight_points,
        false_win: rules.false_win && !rules.minimum_eight_points,
        ..rules
    };
    add_mahjong_rule_config_row(
        commands,
        rules_panel,
        MahjongRuleConfigRow {
            label: "8 番起和",
            value: if rules.minimum_eight_points {
                "开启"
            } else {
                "关闭"
            }
            .to_owned(),
            help: "开启时严格按 2014 国标要求至少 8 番（花牌不计入起和）；关闭后只按实际番数结算，不另加 8 分。",
            editable: can_configure,
            previous: can_configure.then_some(minimum_toggled),
            next: can_configure.then_some(minimum_toggled),
        },
        assets,
    );
    let winners_toggled = MahjongRuleSet {
        multiple_winners: !rules.multiple_winners,
        ..rules
    };
    add_mahjong_rule_config_row(
        commands,
        rules_panel,
        MahjongRuleConfigRow {
            label: "一炮多响",
            value: if rules.multiple_winners {
                "允许"
            } else {
                "截和"
            }
            .to_owned(),
            help: "关闭时按出牌者之后的座次由最近一家截和；开启时所有合法和牌同时结算。",
            editable: can_configure,
            previous: can_configure.then_some(winners_toggled),
            next: can_configure.then_some(winners_toggled),
        },
        assets,
    );
    let false_win_toggled = MahjongRuleSet {
        false_win: !rules.false_win,
        ..rules
    };
    add_mahjong_rule_config_row(
        commands,
        rules_panel,
        MahjongRuleConfigRow {
            label: "允许错和",
            value: if !rules.minimum_eight_points {
                "不适用"
            } else if rules.false_win {
                "开启"
            } else {
                "关闭"
            }
            .to_owned(),
            help: "开启后牌型已经完整便显示和牌按钮；不足 8 番属于错和，向其余三家各付 10 分并公开手牌，本盘继续。",
            editable: can_configure,
            previous: (can_configure && rules.minimum_eight_points).then_some(false_win_toggled),
            next: (can_configure && rules.minimum_eight_points).then_some(false_win_toggled),
        },
        assets,
    );

    let players_panel = add_panel(
        commands,
        content,
        Node {
            min_width: px(500),
            flex_basis: px(650),
            flex_grow: 2.0,
            flex_direction: FlexDirection::Column,
            row_gap: px(11),
            ..default()
        },
        PANEL_ALT,
        PanelSkin::Section,
        assets,
    );
    add_section_title(
        commands,
        players_panel,
        format!("玩家席位  {connected}/4"),
        assets,
    );
    render_seat_selector(commands, players_panel, client, lobby, assets, avatars);
    let actions = spawn_node(
        commands,
        players_panel,
        Node {
            width: percent(100),
            min_height: px(48),
            flex_direction: FlexDirection::Row,
            column_gap: px(12),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::FlexEnd,
            ..default()
        },
        None,
    );
    let you = client.0.model().you();
    let ready = you
        .and_then(|you| lobby.players.iter().find(|player| player.id == you))
        .is_some_and(|player| player.ready);
    let is_host = you == lobby.host;
    add_action_button(
        commands,
        actions,
        "退出房间",
        UiAction::LeaveRoom,
        ButtonKind::Pass,
        assets,
    );
    if is_host {
        let can_start = connected == 4
            && lobby
                .players
                .iter()
                .filter(|player| player.connected)
                .all(|player| player.seat.is_some() && player.ready);
        if can_start {
            add_action_button(
                commands,
                actions,
                "开始游戏",
                UiAction::StartGame,
                ButtonKind::Primary,
                assets,
            );
        } else {
            add_disabled_action_button(commands, actions, "等待四名玩家", assets);
        }
    } else {
        add_action_button(
            commands,
            actions,
            if ready { "取消准备" } else { "准备" },
            UiAction::ToggleReady,
            if ready {
                ButtonKind::Secondary
            } else {
                ButtonKind::Primary
            },
            assets,
        );
    }
}

pub(in crate::app) fn render_mahjong_table(
    commands: &mut Commands,
    root: Entity,
    client: &ClientResource,
    game: &MahjongSnapshot,
    ui: &mut UiState,
    chat: &ChatPanelState,
    interaction_menu_open: Option<PlayerId>,
    visuals: MahjongTableVisuals<'_>,
) {
    let MahjongTableVisuals {
        assets,
        avatars,
        appearance,
        brightness,
        vignette,
        table_materials,
        tile_materials,
        game_summary,
        claim_presentation,
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
    let material = table_materials.add(TableBackgroundMaterial {
        params: table_material_params(brightness, vignette, appearance.custom_felt.is_none()),
        texture: felt,
    });
    commands
        .entity(content)
        .insert((MaterialNode(material), TableBackground));
    let table = spawn_node(
        commands,
        content,
        Node {
            position_type: PositionType::Absolute,
            left: percent(50),
            top: px(0),
            bottom: px(0),
            width: px(DESIGN_WIDTH),
            min_height: px(430),
            ..default()
        },
        None,
    );
    commands.entity(table).insert(UiTransform {
        translation: Val2::px(-DESIGN_WIDTH / 2.0, 0.0),
        ..default()
    });

    let own_seat = game
        .players
        .iter()
        .find(|player| player.id == game.you)
        .map(|player| player.seat.0)
        .unwrap_or_default();
    let new_hand = ui.mahjong_observed_match != Some(game.match_id)
        || ui.mahjong_observed_sequence != game.sequence_index;
    if new_hand {
        ui.mahjong_observed_hand.clear();
        ui.mahjong_observed_counts = [0; 4];
        ui.mahjong_observed_flowers = [0; 4];
    }
    let received_batch = game.players.iter().any(|player| {
        let index = player.id.0 as usize;
        player.concealed_count > ui.mahjong_observed_counts[index]
            || player.flowers.len() > usize::from(ui.mahjong_observed_flowers[index])
    });
    let dealing = matches!(
        game.phase,
        MahjongPhaseView::Dealing { .. } | MahjongPhaseView::ReplacingFlower { .. }
    );
    let winners = match &game.phase {
        MahjongPhaseView::Finished { result } => Some(&result.winners),
        _ => None,
    };
    render_mahjong_wall(commands, table, game.wall_len, assets, tile_materials);
    render_discard_rivers(commands, table, game, own_seat, assets, tile_materials);
    for player in &game.players {
        let winning_hand =
            winners.is_some_and(|winners| winners.iter().any(|winner| winner.player == player.id));
        let flower_replaced =
            player.flowers.len() > usize::from(ui.mahjong_observed_flowers[player.id.0 as usize]);
        let separate_last_concealed = matches!(
            game.phase,
            MahjongPhaseView::Playing | MahjongPhaseView::ReplacingFlower { .. }
        ) && game.current_player == player.id
            && player.concealed_count % 3 == 2;
        let active_claim = claim_presentation
            .active
            .as_ref()
            .filter(|claim| claim.match_id == game.match_id && claim.player == player.id);
        render_mahjong_player_tiles(
            commands,
            table,
            player,
            own_seat,
            ui.mahjong_observed_counts[player.id.0 as usize],
            flower_replaced,
            dealing || flower_replaced,
            winning_hand,
            separate_last_concealed,
            game_summary,
            active_claim,
            assets,
            tile_materials,
        );
        render_mahjong_player_panel(
            commands,
            table,
            game,
            player,
            own_seat,
            interaction_menu_open,
            assets,
            avatars,
        );
    }
    render_round_status(commands, table, game, own_seat, assets);
    let own_flower_replaced = game
        .players
        .iter()
        .find(|player| player.id == game.you)
        .is_some_and(|player| {
            player.flowers.len() > usize::from(ui.mahjong_observed_flowers[player.id.0 as usize])
        });
    render_own_hand(
        commands,
        table,
        game,
        &ui.mahjong_observed_hand,
        dealing || own_flower_replaced,
        winners.is_some_and(|winners| winners.iter().any(|winner| winner.player == game.you)),
        game_summary,
        claim_presentation.active.as_ref().filter(|claim| {
            claim.match_id == game.match_id && claim.player == game.you && claim.shift_hand
        }),
        assets,
        tile_materials,
    );
    if received_batch {
        queue_mahjong_deal_sound(commands, assets);
    }
    render_action_bar(commands, table, game, assets);
    render_mahjong_claim_presentation(
        commands,
        table,
        game,
        own_seat,
        claim_presentation,
        assets,
        tile_materials,
    );
    render_mahjong_flower_presentations(
        commands,
        table,
        game,
        own_seat,
        claim_presentation,
        assets,
    );
    render_mahjong_win_effects(commands, table, game, own_seat, game_summary, assets);
    add_chat_panel(commands, content, chat, assets, None, None, None);
    if let MahjongPhaseView::Finished { result } = &game.phase {
        render_mahjong_settlement(commands, table, game, result, assets, avatars, game_summary);
    }
    ui.mahjong_observed_match = Some(game.match_id);
    ui.mahjong_observed_sequence = game.sequence_index;
    ui.mahjong_observed_hand.clone_from(&game.your_hand);
    for player in &game.players {
        ui.mahjong_observed_counts[player.id.0 as usize] = player.concealed_count;
        ui.mahjong_observed_flowers[player.id.0 as usize] = player.flowers.len() as u8;
    }
    let _ = client;
}

fn render_round_status(
    commands: &mut Commands,
    table: Entity,
    game: &MahjongSnapshot,
    own_seat: u8,
    assets: &UiAssets,
) {
    let status = add_panel(
        commands,
        table,
        Node {
            position_type: PositionType::Absolute,
            left: px(565),
            top: px(288),
            width: px(150),
            height: px(88),
            border: UiRect::all(px(1)),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            flex_direction: FlexDirection::Column,
            row_gap: px(4),
            ..default()
        },
        Color::srgba(0.018, 0.075, 0.052, 0.94),
        PanelSkin::Section,
        assets,
    );
    commands.entity(status).insert((
        BorderColor::all(Color::srgba(0.72, 0.58, 0.25, 0.72)),
        BoxShadow::new(Color::BLACK.with_alpha(0.46), px(0), px(4), px(0), px(9)),
        ZIndex(20),
    ));
    add_text(
        commands,
        status,
        format!(
            "{}风  第 {} 局",
            wind_label(game.prevalent_wind),
            game.sequence_index + 1
        ),
        15.0,
        Color::srgb(0.92, 0.79, 0.43),
        assets,
    );
    add_text(
        commands,
        status,
        game.wall_len.to_string(),
        25.0,
        TEXT,
        assets,
    );
    add_text(commands, status, "牌墙余张", 10.0, MUTED, assets);
    if matches!(game.phase, MahjongPhaseView::Playing) {
        let current_seat = game
            .players
            .iter()
            .find(|player| player.id == game.current_player)
            .map(|player| player.seat.0)
            .unwrap_or(own_seat);
        render_turn_arrows(commands, table, (current_seat + 4 - own_seat) % 4, assets);
    }
}

fn render_turn_arrows(commands: &mut Commands, table: Entity, relative: u8, assets: &UiAssets) {
    let (origin, direction, rotation) = match relative {
        0 => (
            Vec2::new(630.0, 382.0),
            Vec2::new(0.0, 15.0),
            std::f32::consts::FRAC_PI_2,
        ),
        1 => (Vec2::new(720.0, 322.0), Vec2::new(15.0, 0.0), 0.0),
        2 => (
            Vec2::new(630.0, 266.0),
            Vec2::new(0.0, -15.0),
            -std::f32::consts::FRAC_PI_2,
        ),
        _ => (
            Vec2::new(540.0, 322.0),
            Vec2::new(-15.0, 0.0),
            std::f32::consts::PI,
        ),
    };
    for slot in 0..3 {
        let position = origin + direction * slot as f32;
        let arrow = commands
            .spawn((
                Node {
                    position_type: PositionType::Absolute,
                    left: px(position.x),
                    top: px(position.y),
                    width: px(20),
                    height: px(20),
                    ..default()
                },
                ImageNode::new(assets.mahjong_turn_arrow.clone())
                    .with_color(Color::WHITE.with_alpha(0.0)),
                UiTransform::from_rotation(Rot2::radians(rotation)),
                MahjongTurnArrow { slot: slot as f32 },
                ZIndex(24),
                FocusPolicy::Pass,
            ))
            .id();
        commands.entity(table).add_child(arrow);
    }
}

fn mahjong_deal_spec(
    relative: u8,
    tile_index: usize,
    tile_count: usize,
    advance: f32,
) -> MahjongDealSpec {
    let final_x = (tile_index as f32 - (tile_count.saturating_sub(1)) as f32 * 0.5) * advance;
    let toward_wall = match relative {
        0 => -235.0,
        1 | 3 => 205.0,
        2 => -180.0,
        _ => -180.0,
    };
    MahjongDealSpec {
        start_offset: Vec2::new(-final_x, toward_wall),
        start_rotation: match relative {
            1 => -0.12,
            3 => 0.12,
            _ => ((tile_index * 19 % 5) as f32 - 2.0) * 0.025,
        },
    }
}

fn queue_mahjong_deal_sound(commands: &mut Commands, assets: &UiAssets) {
    if assets.deal_sounds.is_empty() {
        return;
    }
    commands.spawn(PendingDealSound {
        remaining: 0.0,
        variant: fastrand::usize(..assets.deal_sounds.len()),
    });
}

fn render_mahjong_player_panel(
    commands: &mut Commands,
    table: Entity,
    game: &MahjongSnapshot,
    player: &leocard_protocol::MahjongPlayerState,
    own_seat: u8,
    interaction_menu_open: Option<PlayerId>,
    assets: &UiAssets,
    avatars: &AvatarImages,
) {
    let relative = (player.seat.0 + 4 - own_seat) % 4;
    let (left, top, bottom, width) = match relative {
        0 => (8.0, None, Some(8.0), 165.0),
        1 => (1107.0, Some(300.0), None, 165.0),
        2 => (557.5, Some(8.0), None, 165.0),
        _ => (8.0, Some(300.0), None, 165.0),
    };
    let panel = commands
        .spawn((
            Button,
            UiAction::ToggleInteractionMenu(player.id),
            Node {
                position_type: PositionType::Absolute,
                left: px(left),
                top: top.map_or(Val::Auto, |value| px(value)),
                bottom: bottom.map_or(Val::Auto, |value| px(value)),
                width: px(width),
                height: px(50),
                padding: UiRect::all(px(6)),
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::Center,
                row_gap: px(3),
                border: UiRect::all(px(1)),
                border_radius: BorderRadius::all(px(8)),
                ..default()
            },
            BackgroundColor(PANEL.with_alpha(0.92)),
            BorderColor::all(BORDER),
            BoxShadow::new(Color::BLACK.with_alpha(0.32), px(0), px(3), px(0), px(7)),
        ))
        .id();
    commands.entity(table).add_child(panel);
    let head = spawn_node(
        commands,
        panel,
        Node {
            width: percent(100),
            align_items: AlignItems::Center,
            column_gap: px(7),
            ..default()
        },
        None,
    );
    let avatar = player.avatar.and_then(|id| avatars.remote.get(&id));
    let avatar_entity = add_avatar(commands, head, &player.name, avatar, 30.0, assets);
    commands
        .entity(avatar_entity)
        .insert(PlayerAvatarAnchor(player.id));
    let info = spawn_node(
        commands,
        head,
        Node {
            flex_grow: 1.0,
            flex_direction: FlexDirection::Column,
            ..default()
        },
        None,
    );
    add_text(
        commands,
        info,
        format!(
            "{}  {}{}",
            player.name,
            wind_label(player.seat_wind),
            if player.id == game.dealer { "庄" } else { "" }
        ),
        14.0,
        if player.dead_hand { DANGER } else { TEXT },
        assets,
    );
    add_text(
        commands,
        info,
        format!(
            "花 {} · 累计 {:+}",
            player.flowers.len(),
            game.match_scores[player.id.0 as usize]
        ),
        10.0,
        MUTED,
        assets,
    );
    let side = match relative {
        1 => SeatSide::Right,
        2 => SeatSide::Top,
        _ => SeatSide::Left,
    };
    let menu = add_interaction_menu(
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
        .entity(menu)
        .insert(if interaction_menu_open == Some(player.id) {
            Visibility::Visible
        } else {
            Visibility::Hidden
        });
    commands.entity(panel).insert(OpponentBadge {
        player: player.id,
        score_popup: None,
        interaction_menu: menu,
    });
}

fn render_mahjong_wall(
    commands: &mut Commands,
    table: Entity,
    wall_len: u16,
    assets: &UiAssets,
    materials: &mut Assets<MahjongTileMaterial>,
) {
    let stack_count = usize::from(wall_len).div_ceil(2).min(72);
    for side in 0..4 {
        let count = stack_count.saturating_sub(side * 18).min(18);
        let (left, top, rotation) = match side {
            0 => (460.0, 135.0, 0.0),
            1 => (820.0, 302.0, std::f32::consts::FRAC_PI_2),
            2 => (460.0, 480.0, std::f32::consts::PI),
            _ => (100.0, 302.0, -std::f32::consts::FRAC_PI_2),
        };
        let segment = spawn_node(
            commands,
            table,
            Node {
                position_type: PositionType::Absolute,
                left: px(left),
                top: px(top),
                width: px(360),
                height: px(46),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::FlexStart,
                flex_direction: FlexDirection::Row,
                ..default()
            },
            None,
        );
        commands.entity(segment).insert((
            UiTransform::from_rotation(Rot2::radians(rotation)),
            ZIndex(1),
        ));
        for index in 0..count {
            let wall_stack_index = side * 18 + index;
            let double = wall_stack_index * 2 + 1 < usize::from(wall_len);
            let layers = if double { 2 } else { 1 };
            let stack = spawn_node(
                commands,
                segment,
                Node {
                    width: px(20),
                    min_width: px(20),
                    height: px(44),
                    ..default()
                },
                None,
            );
            commands.entity(stack).insert(ZIndex(index as i32));
            let orientation = match side {
                1 => 3,
                3 => 1,
                _ => side as u8,
            };
            add_mahjong_wall_stack(commands, stack, layers, orientation, assets, materials);
        }
    }
}

fn add_mahjong_wall_stack(
    commands: &mut Commands,
    stack: Entity,
    layers: u8,
    orientation: u8,
    assets: &UiAssets,
    materials: &mut Assets<MahjongTileMaterial>,
) {
    let material = materials.add(MahjongTileMaterial {
        params: Vec4::new(0.0, 1.0, 1.0, if layers == 2 { 2.0 } else { 3.0 }),
        lighting: mahjong_local_light(orientation),
        glyph: assets.mahjong_tile_back.clone(),
        height: assets.mahjong_tile_back.clone(),
    });
    let shadow = mahjong_local_shadow(orientation);
    let tile = commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                left: px(0),
                top: px(0),
                width: px(26),
                height: px(46),
                border_radius: BorderRadius::all(px(3)),
                overflow: Overflow::visible(),
                ..default()
            },
            MaterialNode(material),
            BoxShadow::new(
                Color::BLACK.with_alpha(0.10),
                px(shadow.x),
                px(shadow.y),
                px(0),
                px(2),
            ),
            ZIndex(1),
            FocusPolicy::Pass,
        ))
        .id();
    commands.entity(stack).add_child(tile);
}

fn render_mahjong_player_tiles(
    commands: &mut Commands,
    table: Entity,
    player: &leocard_protocol::MahjongPlayerState,
    own_seat: u8,
    observed_count: u8,
    flower_replaced: bool,
    dealing: bool,
    winning_hand: bool,
    separate_last_concealed: bool,
    animation: &GameSummaryAnimation,
    active_claim: Option<&ActiveMahjongClaimPresentation>,
    assets: &UiAssets,
    materials: &mut Assets<MahjongTileMaterial>,
) {
    let relative = (player.seat.0 + 4 - own_seat) % 4;
    if relative == 0 && player.melds.is_empty() {
        return;
    }
    let (left, top, bottom, width, height, rotation) = match relative {
        0 => (MAHJONG_OWN_HAND_LEFT, None, Some(8.0), 760.0, 80.0, 0.0),
        1 => (
            840.0,
            Some(302.0),
            None,
            450.0,
            54.0,
            -std::f32::consts::FRAC_PI_2,
        ),
        2 => (415.0, Some(58.0), None, 450.0, 54.0, std::f32::consts::PI),
        _ => (
            -10.0,
            Some(302.0),
            None,
            450.0,
            54.0,
            std::f32::consts::FRAC_PI_2,
        ),
    };
    let group = spawn_node(
        commands,
        table,
        Node {
            position_type: PositionType::Absolute,
            left: px(left),
            top: top.map_or(Val::Auto, px),
            bottom: bottom.map_or(Val::Auto, px),
            width: px(width),
            height: px(height),
            align_items: AlignItems::FlexEnd,
            justify_content: JustifyContent::FlexStart,
            flex_direction: FlexDirection::Row,
            column_gap: px(0),
            ..default()
        },
        None,
    );
    let mut transform = UiTransform::from_rotation(Rot2::radians(rotation));
    if winning_hand {
        apply_mahjong_winning_hand_visual(
            &mut transform,
            relative,
            rotation,
            mahjong_winning_hand_progress(animation.elapsed),
        );
        commands.entity(group).insert(MahjongWinningHand {
            relative,
            base_rotation: rotation,
        });
    }
    commands.entity(group).insert((transform, ZIndex(8)));

    let staged_claim = active_claim
        .filter(|claim| claim.source.is_some() && claim.elapsed < mahjong_claim_landing_time());
    let mut meld_index = 20;
    let visible_meld_count = player
        .melds
        .len()
        .saturating_sub(usize::from(staged_claim.is_some()));
    for meld in player.melds.iter().take(visible_meld_count) {
        render_mahjong_meld(
            commands,
            group,
            meld,
            &mut meld_index,
            relative,
            assets,
            materials,
        );
    }
    if let Some(claim) = staged_claim {
        render_mahjong_staged_meld(
            commands,
            group,
            claim,
            &mut meld_index,
            relative,
            assets,
            materials,
        );
    }
    if relative != 0 && (visible_meld_count > 0 || staged_claim.is_some()) {
        let gap = spawn_node(
            commands,
            group,
            Node {
                width: px(10),
                min_width: px(10),
                height: px(1),
                ..default()
            },
            None,
        );
        commands.entity(gap).insert(FocusPolicy::Pass);
    }

    if relative != 0 {
        let concealed = spawn_node(
            commands,
            group,
            Node {
                height: px(height),
                align_items: AlignItems::FlexEnd,
                flex_direction: FlexDirection::Row,
                flex_shrink: 0.0,
                overflow: Overflow::visible(),
                ..default()
            },
            None,
        );
        if let Some(claim) = active_claim.filter(|claim| claim.shift_hand) {
            let distance = if player.melds.len() == 1 {
                MAHJONG_REMOTE_MELD_WIDTH + 10.0
            } else {
                MAHJONG_REMOTE_MELD_WIDTH
            };
            commands.entity(concealed).insert((
                MahjongClaimHandShift {
                    player: player.id,
                    distance,
                },
                UiTransform::from_translation(Val2::px(
                    mahjong_claim_hand_shift_x(claim.elapsed, distance),
                    0.0,
                )),
            ));
        }
        let concealed_count = usize::from(player.concealed_count);
        let hidden_size = match relative {
            1 | 3 => MahjongTileSize::HiddenSide,
            _ => MahjongTileSize::HiddenOpposite,
        };
        if let Some(revealed) = &player.revealed_hand {
            for (index, tile) in revealed.iter().enumerate() {
                add_mahjong_tile_material(
                    commands,
                    concealed,
                    Some(tile.kind()),
                    if winning_hand {
                        MahjongTileSize::Mini
                    } else {
                        hidden_size
                    },
                    index,
                    false,
                    (dealing
                        && (index >= usize::from(observed_count)
                            || (flower_replaced && index + 1 == concealed_count)))
                        .then(|| mahjong_deal_spec(relative, index, concealed_count, 24.0)),
                    relative,
                    assets,
                    materials,
                );
            }
        } else {
            let joined_count = concealed_count.saturating_sub(usize::from(separate_last_concealed));
            for index in 0..joined_count {
                add_mahjong_tile_material(
                    commands,
                    concealed,
                    None,
                    hidden_size,
                    index,
                    false,
                    (dealing
                        && (index >= usize::from(observed_count)
                            || (flower_replaced && index + 1 == concealed_count)))
                        .then(|| mahjong_deal_spec(relative, index, concealed_count, 24.0)),
                    relative,
                    assets,
                    materials,
                );
            }
            if separate_last_concealed && concealed_count > 0 {
                let gap = spawn_node(
                    commands,
                    concealed,
                    Node {
                        width: px(14),
                        min_width: px(14),
                        height: px(1),
                        ..default()
                    },
                    None,
                );
                commands.entity(gap).insert(FocusPolicy::Pass);
                add_mahjong_tile_material(
                    commands,
                    concealed,
                    None,
                    hidden_size,
                    concealed_count - 1,
                    false,
                    None,
                    relative,
                    assets,
                    materials,
                );
            }
        }
    }
}

fn render_discard_rivers(
    commands: &mut Commands,
    table: Entity,
    game: &MahjongSnapshot,
    own_seat: u8,
    assets: &UiAssets,
    materials: &mut Assets<MahjongTileMaterial>,
) {
    let last_discard = game
        .discards
        .iter()
        .rposition(|discard| discard.claimed_by.is_none());
    for player in &game.players {
        let relative = (player.seat.0 + 4 - own_seat) % 4;
        let (left, top, rotation) = match relative {
            0 => (549.0, 382.0, 0.0),
            1 => (785.0, 245.0, -std::f32::consts::FRAC_PI_2),
            2 => (533.0, 160.0, std::f32::consts::PI),
            _ => (281.0, 245.0, std::f32::consts::FRAC_PI_2),
        };
        let river = spawn_node(
            commands,
            table,
            Node {
                position_type: PositionType::Absolute,
                left: px(left),
                top: px(top),
                width: px(182),
                height: px(126),
                align_content: AlignContent::FlexStart,
                flex_direction: FlexDirection::Row,
                flex_wrap: FlexWrap::Wrap,
                row_gap: px(-3),
                overflow: Overflow::visible(),
                ..default()
            },
            None,
        );
        commands.entity(river).insert((
            UiTransform::from_rotation(Rot2::radians(rotation)),
            ZIndex(5),
        ));
        for (index, discard) in game
            .discards
            .iter()
            .enumerate()
            .filter(|(_, discard)| discard.player == player.id && discard.claimed_by.is_none())
        {
            add_mahjong_tile_material(
                commands,
                river,
                Some(discard.tile.kind()),
                MahjongTileSize::River,
                index,
                last_discard == Some(index),
                None,
                relative,
                assets,
                materials,
            );
        }
    }
}

fn render_own_hand(
    commands: &mut Commands,
    table: Entity,
    game: &MahjongSnapshot,
    observed_hand: &[MahjongTile],
    dealing: bool,
    winning_hand: bool,
    animation: &GameSummaryAnimation,
    active_claim: Option<&ActiveMahjongClaimPresentation>,
    assets: &UiAssets,
    materials: &mut Assets<MahjongTileMaterial>,
) {
    let meld_count = game
        .players
        .iter()
        .find(|player| player.id == game.you)
        .map_or(0, |player| player.melds.len());
    let left = MAHJONG_OWN_HAND_LEFT + meld_count as f32 * MAHJONG_OWN_MELD_WIDTH;
    let hand = spawn_node(
        commands,
        table,
        Node {
            position_type: PositionType::Absolute,
            left: px(left),
            width: px(760),
            bottom: px(8),
            height: px(80),
            align_items: AlignItems::FlexEnd,
            justify_content: JustifyContent::FlexStart,
            flex_direction: FlexDirection::Row,
            ..default()
        },
        None,
    );
    if let Some(claim) = active_claim {
        commands.entity(hand).insert((
            MahjongClaimHandShift {
                player: game.you,
                distance: MAHJONG_OWN_MELD_WIDTH,
            },
            UiTransform::from_translation(Val2::px(
                mahjong_claim_hand_shift_x(claim.elapsed, MAHJONG_OWN_MELD_WIDTH),
                0.0,
            )),
        ));
    }
    if winning_hand {
        let progress = mahjong_winning_hand_progress(animation.elapsed);
        let mut transform = UiTransform::default();
        apply_mahjong_winning_hand_visual(&mut transform, 0, 0.0, progress);
        commands.entity(hand).insert((
            MahjongWinningHand {
                relative: 0,
                base_rotation: 0.0,
            },
            transform,
        ));
    }
    let can_discard =
        matches!(game.phase, MahjongPhaseView::Playing) && game.current_player == game.you;
    let separated_tile = if dealing {
        game.your_drawn_tile
    } else if can_discard {
        game.your_drawn_tile.or_else(|| {
            (game.your_hand.len() % 3 == 2)
                .then(|| game.your_hand.last().copied())
                .flatten()
        })
    } else {
        None
    };
    let separated_index = separated_tile
        .and_then(|separated| game.your_hand.iter().position(|tile| *tile == separated));
    for (index, tile) in game.your_hand.iter().copied().enumerate() {
        if separated_index == Some(index) {
            continue;
        }
        if winning_hand {
            add_mahjong_tile_material(
                commands,
                hand,
                Some(tile.kind()),
                MahjongTileSize::OwnMeld,
                index,
                false,
                None,
                0,
                assets,
                materials,
            );
        } else {
            add_mahjong_hand_tile(
                commands,
                hand,
                tile.kind(),
                can_discard.then_some(UiAction::MahjongDiscard(tile)),
                index,
                (dealing && !observed_hand.contains(&tile))
                    .then(|| mahjong_deal_spec(0, index, game.your_hand.len(), 50.0)),
                assets,
                materials,
            );
        }
    }
    if let Some(index) = separated_index {
        let gap = spawn_node(
            commands,
            hand,
            Node {
                width: px(18),
                min_width: px(18),
                height: px(1),
                ..default()
            },
            None,
        );
        commands.entity(gap).insert(FocusPolicy::Pass);
        let tile = game.your_hand[index];
        add_mahjong_hand_tile(
            commands,
            hand,
            tile.kind(),
            can_discard.then_some(UiAction::MahjongDiscard(tile)),
            game.your_hand.len(),
            (dealing && !observed_hand.contains(&tile)).then(|| {
                mahjong_deal_spec(
                    0,
                    game.your_hand.len().saturating_sub(1),
                    game.your_hand.len(),
                    50.0,
                )
            }),
            assets,
            materials,
        );
    }
}

fn render_action_bar(
    commands: &mut Commands,
    table: Entity,
    game: &MahjongSnapshot,
    assets: &UiAssets,
) {
    let bar = spawn_node(
        commands,
        table,
        Node {
            position_type: PositionType::Absolute,
            left: px(300),
            right: px(300),
            bottom: px(88),
            min_height: px(48),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            column_gap: px(8),
            ..default()
        },
        None,
    );
    if let Some(pending) = &game.pending_claim {
        if pending.your_response.is_some() {
            add_text(commands, bar, "已响应，等待其他玩家", 15.0, MUTED, assets);
            return;
        }
        if pending.your_options.is_empty() {
            add_text(commands, bar, "等待其他玩家响应", 15.0, MUTED, assets);
            return;
        }
        for option in &pending.your_options {
            let (label, claim) = match *option {
                MahjongClaimOption::Chow { start } => (
                    format!("吃 {start}{}{}", start + 1, start + 2),
                    MahjongClaim::Chow { start },
                ),
                MahjongClaimOption::Pung => ("碰".to_owned(), MahjongClaim::Pung),
                MahjongClaimOption::Kong => ("杠".to_owned(), MahjongClaim::Kong),
                MahjongClaimOption::Win => ("和".to_owned(), MahjongClaim::Win),
            };
            add_action_button(
                commands,
                bar,
                &label,
                UiAction::MahjongRespond(claim),
                if matches!(claim, MahjongClaim::Win) {
                    ButtonKind::Warning
                } else {
                    ButtonKind::Primary
                },
                assets,
            );
        }
        add_action_button(
            commands,
            bar,
            "过",
            UiAction::MahjongRespond(MahjongClaim::Pass),
            ButtonKind::Pass,
            assets,
        );
        return;
    }
    if !matches!(game.phase, MahjongPhaseView::Playing) || game.current_player != game.you {
        return;
    }
    if game.can_self_draw {
        add_action_button(
            commands,
            bar,
            "自摸",
            UiAction::MahjongSelfDraw,
            ButtonKind::Warning,
            assets,
        );
    }
    for kind in &game.concealed_kong_options {
        add_action_button(
            commands,
            bar,
            &format!("暗杠 {}", kind),
            UiAction::MahjongConcealedKong(*kind),
            ButtonKind::Secondary,
            assets,
        );
    }
    for tile in &game.added_kong_options {
        add_action_button(
            commands,
            bar,
            &format!("加杠 {}", tile.kind()),
            UiAction::MahjongAddedKong(*tile),
            ButtonKind::Secondary,
            assets,
        );
    }
}

fn render_mahjong_settlement(
    commands: &mut Commands,
    table: Entity,
    game: &MahjongSnapshot,
    result: &leocard_protocol::MahjongHandResultView,
    assets: &UiAssets,
    avatars: &AvatarImages,
    animation: &GameSummaryAnimation,
) {
    const FAN_INTERVAL: f32 = 0.14;

    let modal_visual = summary_modal_visual(animation.elapsed);
    let modal = spawn_node(
        commands,
        table,
        Node {
            position_type: PositionType::Absolute,
            left: percent(22),
            right: percent(22),
            top: percent(2),
            min_height: px(390),
            padding: UiRect::all(px(24)),
            flex_direction: FlexDirection::Column,
            row_gap: px(6),
            border_radius: BorderRadius::all(px(12)),
            ..default()
        },
        None,
    );
    commands.entity(modal).insert((
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
    let texture = decorate_panel_skin(commands, modal, PanelSkin::Window, assets);
    commands.entity(texture).insert(GameSummaryPanelTexture);
    add_animated_summary_text(
        commands,
        modal,
        "本局结算",
        26.0,
        ACCENT,
        0.0,
        animation.elapsed,
        assets,
    );
    if result.exhaustive_draw {
        add_animated_summary_text(
            commands,
            modal,
            "本局荒牌",
            14.0,
            MUTED,
            0.0,
            animation.elapsed,
            assets,
        );
    }

    let mut next_delay = SUMMARY_ROW_START_DELAY;
    for winner in &result.winners {
        let name = game
            .players
            .iter()
            .find(|player| player.id == winner.player)
            .map(|player| player.name.as_str())
            .unwrap_or("玩家");
        let outcome = winner.from.map_or_else(
            || format!("{name} 自摸 {}番", winner.score.total_points),
            |from| {
                let source = game
                    .players
                    .iter()
                    .find(|player| player.id == from)
                    .map(|player| player.name.as_str())
                    .unwrap_or("玩家");
                format!("{source} 放炮给 {name} {}番", winner.score.total_points)
            },
        );
        add_animated_summary_text(
            commands,
            modal,
            outcome,
            17.0,
            READY,
            next_delay,
            animation.elapsed,
            assets,
        );
        next_delay += FAN_INTERVAL;
        let fans = spawn_node(
            commands,
            modal,
            Node {
                width: percent(100),
                min_height: px(30),
                flex_direction: FlexDirection::Row,
                flex_wrap: FlexWrap::Wrap,
                align_items: AlignItems::Center,
                column_gap: px(6),
                row_gap: px(6),
                ..default()
            },
            None,
        );
        for fan in &winner.score.fans {
            let delay = next_delay;
            let progress = summary_row_progress(animation.elapsed, delay);
            let color = if fan.points >= 48 {
                Color::srgb(0.98, 0.76, 0.25)
            } else if fan.points >= 8 {
                READY
            } else {
                TEXT
            };
            let badge = spawn_node(
                commands,
                fans,
                Node {
                    min_height: px(28),
                    padding: UiRect::axes(px(9), px(4)),
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    border: UiRect::all(px(1)),
                    border_radius: BorderRadius::all(px(6)),
                    ..default()
                },
                Some(PANEL_ALT.with_alpha(0.82 * progress)),
            );
            commands.entity(badge).insert((
                GameSummaryRow { delay },
                BorderColor::all(color.with_alpha(0.72)),
                UiTransform::from_translation(Val2::px(0.0, 12.0 * (1.0 - progress))),
                if progress > 0.0 {
                    Visibility::Visible
                } else {
                    Visibility::Hidden
                },
            ));
            let fan_name = if fan.count > 1 {
                format!("{}×{}", fan.fan.name(), fan.count)
            } else {
                fan.fan.name().to_owned()
            };
            let text = add_animated_summary_text(
                commands,
                badge,
                format!("{fan_name}  {}番", fan.points),
                if fan.points >= 48 { 14.5 } else { 13.0 },
                color,
                delay,
                animation.elapsed,
                assets,
            );
            if fan.points >= 48 {
                commands.entity(text).insert(TextShadow {
                    offset: Vec2::new(0.8, 0.0),
                    color: color.with_alpha(0.82),
                });
            }
            next_delay += FAN_INTERVAL;
        }
        next_delay += 0.08;
    }

    let list = spawn_node(
        commands,
        modal,
        Node {
            width: percent(100),
            flex_direction: FlexDirection::Column,
            row_gap: px(4),
            ..default()
        },
        None,
    );
    let mut players = game.players.iter().collect::<Vec<_>>();
    players.sort_by(|left, right| {
        result.match_scores[right.id.0 as usize]
            .cmp(&result.match_scores[left.id.0 as usize])
            .then_with(|| left.seat.0.cmp(&right.seat.0))
    });
    for (index, player) in players.iter().enumerate() {
        let delay = next_delay + index as f32 * SUMMARY_ROW_INTERVAL;
        let progress = summary_row_progress(animation.elapsed, delay);
        let row = spawn_node(
            commands,
            list,
            Node {
                width: percent(100),
                min_height: px(38),
                padding: UiRect::axes(px(10), px(4)),
                align_items: AlignItems::Center,
                column_gap: px(10),
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
                ..default()
            },
            None,
        );
        add_animated_summary_text(
            commands,
            name,
            &player.name,
            16.0,
            TEXT,
            delay,
            animation.elapsed,
            assets,
        );
        let delta = result.deltas[player.id.0 as usize];
        let total = result.match_scores[player.id.0 as usize];
        let total_text = add_animated_summary_text(
            commands,
            row,
            format!("累计 {total:+}"),
            16.0,
            ACCENT,
            delay,
            animation.elapsed,
            assets,
        );
        commands
            .entity(total_text)
            .insert(AnimatedSignedSummaryScore {
                target: total,
                delay,
            });
        add_animated_summary_text(
            commands,
            row,
            format!("{delta:+}"),
            16.0,
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
    let actions_delay =
        next_delay + players.len() as f32 * SUMMARY_ROW_INTERVAL + SUMMARY_ACTIONS_EXTRA_DELAY;
    let actions = spawn_node(
        commands,
        modal,
        Node {
            width: percent(100),
            min_height: px(48),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::SpaceBetween,
            column_gap: px(10),
            ..default()
        },
        None,
    );
    commands.entity(actions).insert((
        GameSummaryActions {
            delay: actions_delay,
        },
        if animation.elapsed >= actions_delay {
            Visibility::Visible
        } else {
            Visibility::Hidden
        },
    ));
    add_action_button(
        commands,
        actions,
        "退出游戏",
        UiAction::LeaveRoom,
        ButtonKind::Pass,
        assets,
    );
    if game.you == game.host {
        add_action_button(
            commands,
            actions,
            "返回房间",
            UiAction::ReturnToLobby,
            ButtonKind::Warning,
            assets,
        );
    }
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
            "再来一局",
            UiAction::PlayAgain,
            ButtonKind::Primary,
            assets,
        );
    }
}

fn add_mahjong_hand_tile(
    commands: &mut Commands,
    parent: Entity,
    kind: MahjongTileKind,
    action: Option<UiAction>,
    index: usize,
    deal: Option<MahjongDealSpec>,
    assets: &UiAssets,
    materials: &mut Assets<MahjongTileMaterial>,
) -> Entity {
    let glyph = assets
        .mahjong_tiles
        .get(&kind)
        .cloned()
        .expect("所有麻将牌面都应预加载");
    let height = assets
        .mahjong_tile_heights
        .get(&kind)
        .cloned()
        .expect("所有麻将凹刻高度图都应预加载");
    let material = materials.add(MahjongTileMaterial {
        params: Vec4::new(0.0, 0.0, if deal.is_some() { 0.0 } else { 1.0 }, -1.0),
        lighting: mahjong_local_light(0),
        glyph,
        height,
    });
    let base_rotation = 0.0;
    let entity = commands
        .spawn((
            MahjongHandTile {
                lift: 0.0,
                base_rotation,
                index: index as i32,
            },
            Node {
                width: px(50),
                height: px(68),
                min_width: px(50),
                margin: UiRect::right(px(-5)),
                overflow: Overflow::visible(),
                ..default()
            },
            MaterialNode(material),
            BoxShadow::new(Color::BLACK.with_alpha(0.0), px(1), px(7), px(0), px(3)),
            ZIndex(index as i32),
            UiTransform::from_rotation(Rot2::radians(base_rotation)),
        ))
        .id();
    if let Some(action) = action {
        commands.entity(entity).insert((Button, action));
    }
    if let Some(deal) = deal {
        commands.entity(entity).insert(MahjongDealTile {
            elapsed: 0.0,
            start_offset: deal.start_offset,
            start_rotation: deal.start_rotation,
            final_offset: Vec2::ZERO,
            final_rotation: base_rotation,
            final_shadow_alpha: 0.0,
        });
    }
    commands.entity(parent).add_child(entity);
    entity
}

pub(in crate::app) fn sync_mahjong_hand_tile_materials(
    time: Res<Time>,
    mut materials: ResMut<Assets<MahjongTileMaterial>>,
    mut tiles: Query<
        (
            &mut MahjongHandTile,
            Option<&Interaction>,
            &MaterialNode<MahjongTileMaterial>,
            &mut UiTransform,
            &mut BoxShadow,
            &mut ZIndex,
        ),
        Without<MahjongDealTile>,
    >,
) {
    let smoothing = 1.0 - (-18.0 * time.delta_secs()).exp();
    for (mut tile, interaction, material_node, mut transform, mut shadow, mut z_index) in &mut tiles
    {
        let interaction = interaction.copied().unwrap_or(Interaction::None);
        let target = match interaction {
            Interaction::None => 0.0,
            Interaction::Hovered => 1.0,
            Interaction::Pressed => 0.62,
        };
        tile.lift += (target - tile.lift) * smoothing;
        transform.translation = Val2::px(0.0, -11.0 * tile.lift);
        transform.scale = Vec2::splat(1.0 + 0.025 * tile.lift);
        transform.rotation = Rot2::radians(tile.base_rotation * (1.0 - tile.lift));
        *z_index = ZIndex(if tile.lift > 0.05 {
            100 + tile.index
        } else {
            tile.index
        });
        *shadow = BoxShadow::new(
            Color::BLACK.with_alpha(tile.lift * 0.22),
            px(1),
            px(7.0 + tile.lift * 6.0),
            px(0),
            px(3.0 + tile.lift * 4.0),
        );
        let Some(mut material) = materials.get_mut(&material_node.0) else {
            continue;
        };
        material.params.x = match interaction {
            Interaction::None => 0.0,
            Interaction::Hovered => 1.0,
            Interaction::Pressed => 2.0,
        };
    }
}

pub(in crate::app) fn animate_mahjong_deal_tiles(
    mut commands: Commands,
    time: Res<Time>,
    mut materials: ResMut<Assets<MahjongTileMaterial>>,
    mut tiles: Query<(
        Entity,
        &mut MahjongDealTile,
        &MaterialNode<MahjongTileMaterial>,
        &mut UiTransform,
        &mut BoxShadow,
    )>,
) {
    for (entity, mut deal, material_node, mut transform, mut shadow) in &mut tiles {
        deal.elapsed += time.delta_secs();
        let raw = (deal.elapsed / MAHJONG_DEAL_MOVE_DURATION).clamp(0.0, 1.0);
        let movement = ease_out_cubic(raw);
        let lift = (raw * std::f32::consts::PI).sin() * 12.0;
        let offset = deal.final_offset + deal.start_offset * (1.0 - movement);
        transform.translation = Val2::px(offset.x, offset.y - lift);
        transform.rotation =
            Rot2::radians(deal.final_rotation + deal.start_rotation * (1.0 - movement));
        transform.scale = Vec2::splat(0.82 + movement * 0.18);
        if let Some(mut material) = materials.get_mut(&material_node.0) {
            material.params.z = (raw * 4.0).min(1.0);
        }
        if let Some(style) = shadow.0.first_mut() {
            style.color = Color::BLACK.with_alpha(deal.final_shadow_alpha * raw);
        }
        if raw >= 1.0 {
            commands.entity(entity).remove::<MahjongDealTile>();
        }
    }
}

pub(in crate::app) fn animate_mahjong_turn_arrows(
    time: Res<Time>,
    mut arrows: Query<(&MahjongTurnArrow, &mut ImageNode)>,
) {
    for (arrow, mut image) in &mut arrows {
        let phase = (time.elapsed_secs() * 2.15 - arrow.slot * 0.22).rem_euclid(1.35);
        let alpha = if phase < 0.62 {
            (phase / 0.62 * std::f32::consts::PI).sin().powf(0.72)
        } else {
            0.0
        };
        image.color = Color::WHITE.with_alpha(alpha * 0.92);
    }
}

fn mahjong_claim_label(claim: MahjongClaim) -> &'static str {
    match claim {
        MahjongClaim::Chow { .. } => "吃",
        MahjongClaim::Pung => "碰",
        MahjongClaim::Kong => "杠",
        MahjongClaim::Pass | MahjongClaim::Win => "",
    }
}

fn mahjong_relative_player(game: &MahjongSnapshot, own_seat: u8, player: PlayerId) -> Option<u8> {
    game.players
        .iter()
        .find(|candidate| candidate.id == player)
        .map(|candidate| (candidate.seat.0 + 4 - own_seat) % 4)
}

fn mahjong_claim_river_anchor(relative: u8) -> Vec2 {
    match relative {
        0 => Vec2::new(640.0, 430.0),
        1 => Vec2::new(805.0, 335.0),
        2 => Vec2::new(640.0, 215.0),
        _ => Vec2::new(475.0, 335.0),
    }
}

fn mahjong_claim_player_anchor(
    relative: u8,
    meld_count: usize,
    claim: MahjongClaim,
    tile: MahjongTileKind,
) -> Vec2 {
    let meld_index = meld_count.saturating_sub(1) as f32;
    let center = match relative {
        0 => Vec2::new(MAHJONG_OWN_HAND_LEFT + 70.0 + meld_index * 140.0, 610.0),
        1 => Vec2::new(1073.5, 516.5 - meld_index * MAHJONG_REMOTE_MELD_WIDTH),
        2 => Vec2::new(827.5 - meld_index * MAHJONG_REMOTE_MELD_WIDTH, 76.5),
        _ => Vec2::new(206.5, 141.5 + meld_index * MAHJONG_REMOTE_MELD_WIDTH),
    };
    let slot = match (claim, tile) {
        (MahjongClaim::Chow { start }, MahjongTileKind::Suited { rank, .. }) => {
            i16::from(rank) - i16::from(start) - 1
        }
        _ => 0,
    } as f32;
    let offset = slot * if relative == 0 { 45.0 } else { 24.0 };
    center
        + match relative {
            0 => Vec2::new(offset, 0.0),
            1 => Vec2::new(0.0, -offset),
            2 => Vec2::new(-offset, 0.0),
            _ => Vec2::new(0.0, offset),
        }
}

fn mahjong_claim_angle(relative: u8) -> f32 {
    match relative {
        0 => 0.0,
        1 => -std::f32::consts::FRAC_PI_2,
        2 => std::f32::consts::PI,
        _ => std::f32::consts::FRAC_PI_2,
    }
}

fn mahjong_claim_flight_pose(elapsed: f32, flight: &MahjongClaimFlight) -> (Vec2, f32, f32, f32) {
    let raw = (elapsed / MAHJONG_CLAIM_FLIGHT_DURATION).clamp(0.0, 1.0);
    let progress = ease_out_cubic(raw);
    let inverse = 1.0 - progress;
    let position = flight.start * inverse * inverse
        + flight.control * (2.0 * inverse * progress)
        + flight.target * progress * progress;
    let angle_delta = (flight.target_angle - flight.start_angle + std::f32::consts::PI)
        .rem_euclid(std::f32::consts::TAU)
        - std::f32::consts::PI;
    let angle = flight.start_angle + angle_delta * progress;
    let lift = (raw * std::f32::consts::PI).sin();
    let base_scale = 1.0 + (flight.target_scale - 1.0) * progress;
    (
        position,
        angle,
        base_scale * (1.0 + lift * 0.12),
        (raw / 0.08).min(1.0),
    )
}

fn mahjong_claim_label_visual(elapsed: f32) -> (f32, f32, f32) {
    let focus = (elapsed / 0.24).clamp(0.0, 1.0);
    let focus = ease_out_cubic(focus);
    let scale = 1.0 + (1.0 - focus) * 0.78 + (focus * std::f32::consts::PI).sin() * 0.06;
    let fade_in = (elapsed / 0.08).clamp(0.0, 1.0);
    let fade_out = ((MAHJONG_CLAIM_PRESENTATION_DURATION - elapsed) / 0.32).clamp(0.0, 1.0);
    (scale, fade_in * fade_out, 7.0 * (1.0 - focus))
}

fn mahjong_flower_label_visual(elapsed: f32) -> (f32, f32, f32) {
    let focus = ease_out_cubic((elapsed / 0.24).clamp(0.0, 1.0));
    let scale = 1.0 + (1.0 - focus) * 0.52 + (focus * std::f32::consts::PI).sin() * 0.04;
    let fade_in = (elapsed / 0.08).clamp(0.0, 1.0);
    let fade_out = ((MAHJONG_FLOWER_PRESENTATION_DURATION - elapsed) / 0.28).clamp(0.0, 1.0);
    (scale, fade_in * fade_out, 6.0 * (1.0 - focus))
}

fn mahjong_claim_hand_shift_x(elapsed: f32, distance: f32) -> f32 {
    let progress = ease_out_cubic((elapsed / MAHJONG_CLAIM_HAND_SHIFT_DURATION).clamp(0.0, 1.0));
    -distance * (1.0 - progress)
}

fn mahjong_claim_held_tile_visual(elapsed: f32) -> (f32, f32) {
    let progress = ease_out_cubic((elapsed / 0.24).clamp(0.0, 1.0));
    (0.06 + progress * 0.94, 5.0 * (1.0 - progress))
}

#[allow(clippy::too_many_arguments)]
fn render_mahjong_claim_presentation(
    commands: &mut Commands,
    table: Entity,
    game: &MahjongSnapshot,
    own_seat: u8,
    presentation: &MahjongClaimPresentationState,
    assets: &UiAssets,
    materials: &mut Assets<MahjongTileMaterial>,
) {
    let Some(active) = presentation
        .active
        .as_ref()
        .filter(|active| active.match_id == game.match_id)
    else {
        return;
    };
    let Some(target_relative) = mahjong_relative_player(game, own_seat, active.player) else {
        return;
    };
    if let (Some(source), Some(tile)) = (active.source, active.tile)
        && active.elapsed < mahjong_claim_landing_time()
    {
        let Some(source_relative) = mahjong_relative_player(game, own_seat, source) else {
            return;
        };
        let start = mahjong_claim_river_anchor(source_relative);
        let meld_count = game
            .players
            .iter()
            .find(|player| player.id == active.player)
            .map_or(1, |player| player.melds.len());
        let target =
            mahjong_claim_player_anchor(target_relative, meld_count, active.claim, tile.kind());
        let midpoint = (start + target) * 0.5;
        let toward_center = Vec2::new(640.0, 340.0) - midpoint;
        let flight = MahjongClaimFlight {
            player: active.player,
            tile,
            start,
            control: midpoint + toward_center.normalize_or_zero() * 52.0,
            target,
            start_angle: mahjong_claim_angle(source_relative),
            target_angle: mahjong_claim_angle(target_relative),
            target_scale: if target_relative == 0 {
                50.0 / 33.0
            } else {
                27.0 / 33.0
            },
        };
        let flight_elapsed = (active.elapsed - MAHJONG_CLAIM_FLIGHT_DELAY).max(0.0);
        let (position, angle, scale, _) = mahjong_claim_flight_pose(flight_elapsed, &flight);
        let tile = add_mahjong_tile_material(
            commands,
            table,
            Some(tile.kind()),
            MahjongTileSize::River,
            0,
            false,
            None,
            target_relative,
            assets,
            materials,
        );
        commands.entity(tile).insert((
            flight,
            Node {
                position_type: PositionType::Absolute,
                left: px(position.x - 16.5),
                top: px(position.y - 22.5),
                width: px(33),
                min_width: px(33),
                height: px(45),
                overflow: Overflow::visible(),
                ..default()
            },
            UiTransform {
                rotation: Rot2::radians(angle),
                scale: Vec2::splat(scale),
                ..default()
            },
            BoxShadow::new(Color::BLACK.with_alpha(0.38), px(2), px(6), px(0), px(7)),
            ZIndex(90),
            if active.elapsed >= MAHJONG_CLAIM_FLIGHT_DELAY {
                Visibility::Visible
            } else {
                Visibility::Hidden
            },
        ));
    }

    let label_position = mahjong_claim_river_anchor(target_relative);
    let (scale, alpha, offset_y) = mahjong_claim_label_visual(active.elapsed);
    let holder = spawn_node(
        commands,
        table,
        Node {
            position_type: PositionType::Absolute,
            left: px(label_position.x - 55.0),
            top: px(label_position.y - 34.0),
            width: px(110),
            height: px(68),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            ..default()
        },
        None,
    );
    let text = add_text(
        commands,
        holder,
        mahjong_claim_label(active.claim),
        42.0,
        ACCENT.with_alpha(alpha),
        assets,
    );
    commands.entity(text).insert(TextShadow {
        offset: Vec2::new(1.5, 3.0),
        color: Color::BLACK.with_alpha(0.72 * alpha),
    });
    commands.entity(holder).insert((
        MahjongClaimLabel {
            player: active.player,
            text,
        },
        UiTransform {
            translation: Val2::px(0.0, offset_y),
            scale: Vec2::splat(scale),
            ..default()
        },
        ZIndex(89),
        FocusPolicy::Pass,
    ));
}

fn render_mahjong_flower_presentations(
    commands: &mut Commands,
    table: Entity,
    game: &MahjongSnapshot,
    own_seat: u8,
    presentation: &MahjongClaimPresentationState,
    assets: &UiAssets,
) {
    for active in presentation
        .flowers
        .iter()
        .filter(|active| active.match_id == game.match_id)
    {
        let Some(relative) = mahjong_relative_player(game, own_seat, active.player) else {
            continue;
        };
        let position = mahjong_claim_river_anchor(relative);
        let (scale, alpha, offset_y) = mahjong_flower_label_visual(active.elapsed);
        let holder = spawn_node(
            commands,
            table,
            Node {
                position_type: PositionType::Absolute,
                left: px(position.x - 60.0),
                top: px(position.y - 34.0),
                width: px(120),
                height: px(68),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
            None,
        );
        let text = add_text(
            commands,
            holder,
            "补花",
            38.0,
            TEXT.with_alpha(alpha),
            assets,
        );
        commands.entity(text).insert(TextShadow {
            offset: Vec2::new(1.5, 3.0),
            color: Color::BLACK.with_alpha(0.72 * alpha),
        });
        commands.entity(holder).insert((
            MahjongFlowerLabel {
                player: active.player,
                text,
            },
            UiTransform {
                translation: Val2::px(0.0, offset_y),
                scale: Vec2::splat(scale),
                ..default()
            },
            ZIndex(89),
            FocusPolicy::Pass,
        ));
    }
}

pub(in crate::app) fn animate_mahjong_flower_presentations(
    presentation: Res<MahjongClaimPresentationState>,
    mut labels: Query<(&MahjongFlowerLabel, &mut UiTransform, &mut Visibility)>,
    mut texts: Query<(&mut TextColor, &mut TextShadow)>,
) {
    for (label, mut transform, mut visibility) in &mut labels {
        let Some(active) = presentation
            .flowers
            .iter()
            .find(|active| active.player == label.player)
        else {
            *visibility = Visibility::Hidden;
            continue;
        };
        let (scale, alpha, offset_y) = mahjong_flower_label_visual(active.elapsed);
        transform.translation = Val2::px(0.0, offset_y);
        transform.scale = Vec2::splat(scale);
        if let Ok((mut color, mut shadow)) = texts.get_mut(label.text) {
            color.0 = TEXT.with_alpha(alpha);
            shadow.color = Color::BLACK.with_alpha(0.72 * alpha);
        }
        *visibility = if alpha > 0.0 {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }
}

pub(in crate::app) fn animate_mahjong_claim_presentation(
    presentation: Res<MahjongClaimPresentationState>,
    mut materials: ResMut<Assets<MahjongTileMaterial>>,
    mut flights: Query<
        (
            &MahjongClaimFlight,
            &mut Node,
            &mut UiTransform,
            &MaterialNode<MahjongTileMaterial>,
            &mut Visibility,
        ),
        (
            Without<MahjongClaimLabel>,
            Without<MahjongClaimHandShift>,
            Without<MahjongClaimHeldTile>,
        ),
    >,
    mut labels: Query<
        (&MahjongClaimLabel, &mut UiTransform, &mut Visibility),
        (
            Without<MahjongClaimFlight>,
            Without<MahjongClaimHandShift>,
            Without<MahjongClaimHeldTile>,
        ),
    >,
    mut hands: Query<
        (&MahjongClaimHandShift, &mut UiTransform),
        (
            Without<MahjongClaimFlight>,
            Without<MahjongClaimLabel>,
            Without<MahjongClaimHeldTile>,
        ),
    >,
    mut held_tiles: Query<
        (&MahjongClaimHeldTile, &mut UiTransform, &mut Visibility),
        (
            Without<MahjongClaimFlight>,
            Without<MahjongClaimLabel>,
            Without<MahjongClaimHandShift>,
        ),
    >,
    mut texts: Query<(&mut TextColor, &mut TextShadow)>,
) {
    let active = presentation.active.as_ref();
    for (flight, mut node, mut transform, material_node, mut visibility) in &mut flights {
        let Some(active) = active.filter(|active| {
            active.player == flight.player
                && active.tile == Some(flight.tile)
                && (MAHJONG_CLAIM_FLIGHT_DELAY..mahjong_claim_landing_time())
                    .contains(&active.elapsed)
        }) else {
            *visibility = Visibility::Hidden;
            continue;
        };
        let flight_elapsed = active.elapsed - MAHJONG_CLAIM_FLIGHT_DELAY;
        let (position, angle, scale, alpha) = mahjong_claim_flight_pose(flight_elapsed, flight);
        node.left = px(position.x - 16.5);
        node.top = px(position.y - 22.5);
        transform.rotation = Rot2::radians(angle);
        transform.scale = Vec2::splat(scale);
        if let Some(mut material) = materials.get_mut(&material_node.0) {
            material.params.z = alpha;
        }
        *visibility = Visibility::Visible;
    }
    for (label, mut transform, mut visibility) in &mut labels {
        let Some(active) = active.filter(|active| active.player == label.player) else {
            *visibility = Visibility::Hidden;
            continue;
        };
        let (scale, alpha, offset_y) = mahjong_claim_label_visual(active.elapsed);
        transform.translation = Val2::px(0.0, offset_y);
        transform.scale = Vec2::splat(scale);
        if let Ok((mut color, mut shadow)) = texts.get_mut(label.text) {
            color.0 = ACCENT.with_alpha(alpha);
            shadow.color = Color::BLACK.with_alpha(0.72 * alpha);
        }
        *visibility = if alpha > 0.0 {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }
    for (hand, mut transform) in &mut hands {
        let elapsed = active
            .filter(|active| active.player == hand.player && active.shift_hand)
            .map_or(MAHJONG_CLAIM_HAND_SHIFT_DURATION, |active| active.elapsed);
        transform.translation = Val2::px(mahjong_claim_hand_shift_x(elapsed, hand.distance), 0.0);
    }
    for (tile, mut transform, mut visibility) in &mut held_tiles {
        let Some(active) = active.filter(|active| {
            active.player == tile.player
                && active.source.is_some()
                && active.elapsed < mahjong_claim_landing_time()
        }) else {
            *visibility = Visibility::Hidden;
            continue;
        };
        let (scale_x, offset_y) = mahjong_claim_held_tile_visual(active.elapsed);
        transform.translation = Val2::px(0.0, offset_y);
        transform.scale = Vec2::new(scale_x, 1.0);
        *visibility = Visibility::Visible;
    }
}

fn mahjong_win_effect_color(alpha: f32) -> Color {
    Color::srgb(1.0, 0.74, 0.18).with_alpha(alpha)
}

fn mahjong_win_effect_visual(summary_elapsed: f32) -> Option<(f32, f32, f32)> {
    let elapsed = summary_elapsed + MAHJONG_WIN_REVEAL_DURATION;
    if !(0.0..MAHJONG_WIN_EFFECT_DURATION).contains(&elapsed) {
        return None;
    }
    let focus = ease_out_cubic((elapsed / 0.26).clamp(0.0, 1.0));
    let scale = 1.0 + (1.0 - focus) * 1.05 + (focus * std::f32::consts::PI).sin() * 0.08;
    let fade_in = (elapsed / 0.07).clamp(0.0, 1.0);
    let fade_out = ((MAHJONG_WIN_EFFECT_DURATION - elapsed) / 0.26).clamp(0.0, 1.0);
    Some((scale, fade_in * fade_out, 9.0 * (1.0 - focus)))
}

fn render_mahjong_win_effects(
    commands: &mut Commands,
    table: Entity,
    game: &MahjongSnapshot,
    own_seat: u8,
    animation: &GameSummaryAnimation,
    assets: &UiAssets,
) {
    let MahjongPhaseView::Finished { result } = &game.phase else {
        return;
    };
    let Some((scale, alpha, offset_y)) = mahjong_win_effect_visual(animation.elapsed) else {
        return;
    };
    for winner in &result.winners {
        let Some(relative) = mahjong_relative_player(game, own_seat, winner.player) else {
            continue;
        };
        let position = mahjong_claim_river_anchor(relative);
        let holder = spawn_node(
            commands,
            table,
            Node {
                position_type: PositionType::Absolute,
                left: px(position.x - 55.0),
                top: px(position.y - 43.0),
                width: px(110),
                height: px(86),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
            None,
        );
        let self_draw = winner.from.is_none();
        let text = add_text(
            commands,
            holder,
            if self_draw { "自摸" } else { "和" },
            if self_draw { 44.0 } else { 58.0 },
            mahjong_win_effect_color(alpha),
            assets,
        );
        commands.entity(text).insert(TextShadow {
            offset: Vec2::new(2.0, 4.0),
            color: Color::BLACK.with_alpha(0.76 * alpha),
        });
        commands.entity(holder).insert((
            MahjongWinEffect { text },
            UiTransform {
                translation: Val2::px(0.0, offset_y),
                scale: Vec2::splat(scale),
                ..default()
            },
            ZIndex(95),
            FocusPolicy::Pass,
        ));
    }
}

pub(in crate::app) fn animate_mahjong_win_effects(
    animation: Res<GameSummaryAnimation>,
    mut effects: Query<(&MahjongWinEffect, &mut UiTransform, &mut Visibility)>,
    mut texts: Query<(&mut TextColor, &mut TextShadow)>,
) {
    let visual = mahjong_win_effect_visual(animation.elapsed);
    for (effect, mut transform, mut visibility) in &mut effects {
        let Some((scale, alpha, offset_y)) = visual else {
            *visibility = Visibility::Hidden;
            continue;
        };
        transform.translation = Val2::px(0.0, offset_y);
        transform.scale = Vec2::splat(scale);
        if let Ok((mut color, mut shadow)) = texts.get_mut(effect.text) {
            color.0 = mahjong_win_effect_color(alpha);
            shadow.color = Color::BLACK.with_alpha(0.76 * alpha);
        }
        *visibility = Visibility::Visible;
    }
}

fn mahjong_winning_hand_progress(summary_elapsed: f32) -> f32 {
    ((summary_elapsed + MAHJONG_WIN_REVEAL_DURATION) / MAHJONG_WIN_PUSH_DURATION).clamp(0.0, 1.0)
}

fn apply_mahjong_winning_hand_visual(
    transform: &mut UiTransform,
    relative: u8,
    base_rotation: f32,
    progress: f32,
) {
    let progress = ease_out_cubic(progress);
    let distance = 12.0 * (1.0 - progress);
    let offset = match relative {
        0 => Vec2::new(0.0, distance),
        1 => Vec2::new(distance, 0.0),
        2 => Vec2::new(0.0, -distance),
        _ => Vec2::new(-distance, 0.0),
    };
    transform.translation = Val2::px(offset.x, offset.y);
    transform.rotation = Rot2::radians(base_rotation);
    transform.scale = Vec2::new(0.96 + progress * 0.04, 0.22 + progress * 0.78);
}

pub(in crate::app) fn animate_mahjong_winning_hands(
    animation: Res<GameSummaryAnimation>,
    mut hands: Query<(&MahjongWinningHand, &mut UiTransform)>,
) {
    let progress = mahjong_winning_hand_progress(animation.elapsed);
    for (hand, mut transform) in &mut hands {
        apply_mahjong_winning_hand_visual(
            &mut transform,
            hand.relative,
            hand.base_rotation,
            progress,
        );
    }
}

#[derive(Clone, Copy)]
enum MahjongTileSize {
    River,
    Mini,
    OwnMeld,
    HiddenSide,
    HiddenOpposite,
}

#[allow(clippy::too_many_arguments)]
fn render_mahjong_staged_meld(
    commands: &mut Commands,
    parent: Entity,
    claim: &ActiveMahjongClaimPresentation,
    index: &mut usize,
    relative: u8,
    assets: &UiAssets,
    materials: &mut Assets<MahjongTileMaterial>,
) {
    let Some(claimed_tile) = claim.tile else {
        return;
    };
    let claimed_kind = claimed_tile.kind();
    let base = match claim.claim {
        MahjongClaim::Chow { start } => {
            let MahjongTileKind::Suited { suit, rank } = claimed_kind else {
                return;
            };
            let mut base = [
                Some(MahjongTileKind::suited(suit, start)),
                Some(MahjongTileKind::suited(suit, start + 1)),
                Some(MahjongTileKind::suited(suit, start + 2)),
            ];
            base[usize::from(rank - start)] = None;
            base
        }
        MahjongClaim::Pung => [Some(claimed_kind), None, Some(claimed_kind)],
        MahjongClaim::Kong => [Some(claimed_kind); 3],
        MahjongClaim::Pass | MahjongClaim::Win => return,
    };
    let own_meld = relative == 0;
    let tile_size = if own_meld {
        MahjongTileSize::OwnMeld
    } else {
        MahjongTileSize::Mini
    };
    let (group_width, group_height, tile_advance) = if own_meld {
        (MAHJONG_OWN_MELD_WIDTH, 80.0, 45.0)
    } else {
        (MAHJONG_REMOTE_MELD_WIDTH, 54.0, 24.0)
    };
    let group = spawn_node(
        commands,
        parent,
        Node {
            position_type: PositionType::Relative,
            width: px(group_width),
            min_width: px(group_width),
            height: px(group_height),
            align_items: AlignItems::FlexEnd,
            flex_direction: FlexDirection::Row,
            overflow: Overflow::visible(),
            ..default()
        },
        None,
    );
    let (scale_x, offset_y) = mahjong_claim_held_tile_visual(claim.elapsed);
    for kind in base {
        if let Some(kind) = kind {
            let tile = add_mahjong_tile_material(
                commands,
                group,
                Some(kind),
                tile_size,
                *index,
                false,
                None,
                relative,
                assets,
                materials,
            );
            commands.entity(tile).insert((
                MahjongClaimHeldTile {
                    player: claim.player,
                },
                UiTransform {
                    translation: Val2::px(0.0, offset_y),
                    scale: Vec2::new(scale_x, 1.0),
                    ..default()
                },
            ));
        } else {
            let slot = spawn_node(
                commands,
                group,
                Node {
                    width: px(tile_advance),
                    min_width: px(tile_advance),
                    height: px(1),
                    ..default()
                },
                None,
            );
            commands.entity(slot).insert(FocusPolicy::Pass);
        }
        *index += 1;
    }
}

fn render_mahjong_meld(
    commands: &mut Commands,
    parent: Entity,
    meld: &leocard_protocol::MahjongPublicMeldView,
    index: &mut usize,
    relative: u8,
    assets: &UiAssets,
    materials: &mut Assets<MahjongTileMaterial>,
) {
    let (base, stacked) = match (meld.kind, meld.tile) {
        (MahjongMeldKind::Chow, Some(MahjongTileKind::Suited { suit, rank })) => (
            vec![
                Some(MahjongTileKind::suited(suit, rank)),
                Some(MahjongTileKind::suited(suit, rank + 1)),
                Some(MahjongTileKind::suited(suit, rank + 2)),
            ],
            None,
        ),
        (MahjongMeldKind::Pung, Some(kind)) => (vec![Some(kind); 3], None),
        (MahjongMeldKind::Kong(_), Some(kind)) => (vec![Some(kind); 3], Some(Some(kind))),
        (MahjongMeldKind::Kong(MahjongKongKind::Concealed), None) => (vec![None; 3], Some(None)),
        _ => return,
    };
    let own_meld = relative == 0;
    let tile_size = if own_meld {
        MahjongTileSize::OwnMeld
    } else {
        MahjongTileSize::Mini
    };
    let (group_width, group_height, stack_left, stack_top, stack_width, stack_height) = if own_meld
    {
        (MAHJONG_OWN_MELD_WIDTH, 80.0, 45.0, 7.0, 50.0, 68.0)
    } else {
        (MAHJONG_REMOTE_MELD_WIDTH, 54.0, 24.0, 12.0, 27.0, 37.0)
    };
    let group = spawn_node(
        commands,
        parent,
        Node {
            position_type: PositionType::Relative,
            width: px(group_width),
            min_width: px(group_width),
            height: px(group_height),
            align_items: AlignItems::FlexEnd,
            flex_direction: FlexDirection::Row,
            overflow: Overflow::visible(),
            ..default()
        },
        None,
    );
    for kind in base {
        add_mahjong_tile_material(
            commands, group, kind, tile_size, *index, false, None, relative, assets, materials,
        );
        *index += 1;
    }
    if let Some(kind) = stacked {
        let holder = spawn_node(
            commands,
            group,
            Node {
                position_type: PositionType::Absolute,
                left: px(stack_left),
                top: px(stack_top),
                width: px(stack_width),
                height: px(stack_height),
                overflow: Overflow::visible(),
                ..default()
            },
            None,
        );
        commands.entity(holder).insert(ZIndex(100 + *index as i32));
        let stacked_tile = add_mahjong_tile_material(
            commands, holder, kind, tile_size, *index, false, None, relative, assets, materials,
        );
        let shadow = mahjong_local_shadow(relative) * 1.4;
        commands.entity(stacked_tile).insert(BoxShadow::new(
            Color::BLACK.with_alpha(0.24),
            px(shadow.x),
            px(shadow.y),
            px(0),
            px(4),
        ));
        *index += 1;
    }
}

fn add_mahjong_tile_material(
    commands: &mut Commands,
    parent: Entity,
    kind: Option<MahjongTileKind>,
    size: MahjongTileSize,
    index: usize,
    highlighted: bool,
    deal: Option<MahjongDealSpec>,
    relative: u8,
    assets: &UiAssets,
    materials: &mut Assets<MahjongTileMaterial>,
) -> Entity {
    let (width, height, overlap) = match size {
        MahjongTileSize::River => (33.0, 45.0, -3.0),
        MahjongTileSize::Mini => (27.0, 37.0, -3.0),
        MahjongTileSize::OwnMeld => (50.0, 68.0, -5.0),
        MahjongTileSize::HiddenSide => (34.0, 46.0, -5.0),
        MahjongTileSize::HiddenOpposite => (31.0, 42.0, -4.0),
    };
    let back = kind.is_none();
    let glyph = kind.map_or_else(
        || assets.mahjong_tile_back.clone(),
        |kind| {
            assets
                .mahjong_tiles
                .get(&kind)
                .cloned()
                .expect("所有麻将牌面都应预加载")
        },
    );
    let height_texture = kind.map_or_else(
        || assets.mahjong_tile_back.clone(),
        |kind| {
            assets
                .mahjong_tile_heights
                .get(&kind)
                .cloned()
                .expect("所有麻将凹刻高度图都应预加载")
        },
    );
    let material = materials.add(MahjongTileMaterial {
        params: Vec4::new(
            0.0,
            if back { 1.0 } else { 0.0 },
            if deal.is_some() { 0.0 } else { 1.0 },
            match size {
                MahjongTileSize::HiddenSide => -2.0,
                MahjongTileSize::HiddenOpposite => -3.0,
                MahjongTileSize::OwnMeld => -4.0,
                MahjongTileSize::River | MahjongTileSize::Mini => 0.0,
            },
        ),
        lighting: mahjong_local_light(relative),
        glyph,
        height: height_texture,
    });
    let node = Node {
        width: px(width),
        height: px(height),
        min_width: px(width),
        margin: UiRect::right(px(overlap)),
        border: UiRect::all(px(if highlighted { 1 } else { 0 })),
        border_radius: BorderRadius::all(px(3)),
        overflow: Overflow::visible(),
        ..default()
    };
    let final_rotation = 0.0;
    let final_offset = Vec2::ZERO;
    let final_shadow_alpha = match size {
        MahjongTileSize::River => 0.12,
        MahjongTileSize::Mini => 0.08,
        MahjongTileSize::OwnMeld => 0.12,
        MahjongTileSize::HiddenSide | MahjongTileSize::HiddenOpposite => 0.14,
    };
    let shadow = mahjong_local_shadow(relative);
    let entity = commands
        .spawn((
            node,
            MaterialNode(material),
            BorderColor::all(if highlighted {
                Color::srgba(0.96, 0.78, 0.28, 0.90)
            } else {
                Color::NONE
            }),
            BoxShadow::new(
                if highlighted {
                    Color::srgba(0.95, 0.72, 0.20, 0.35)
                } else {
                    Color::BLACK.with_alpha(if deal.is_some() {
                        0.0
                    } else {
                        final_shadow_alpha
                    })
                },
                px(shadow.x),
                px(shadow.y),
                px(0),
                px(if highlighted { 5 } else { 2 }),
            ),
            ZIndex(index as i32),
            UiTransform {
                translation: Val2::px(final_offset.x, final_offset.y),
                rotation: Rot2::radians(final_rotation),
                ..default()
            },
        ))
        .id();
    if let Some(deal) = deal {
        commands.entity(entity).insert(MahjongDealTile {
            elapsed: 0.0,
            start_offset: deal.start_offset,
            start_rotation: deal.start_rotation,
            final_offset,
            final_rotation,
            final_shadow_alpha,
        });
    }
    commands.entity(parent).add_child(entity);
    entity
}

fn wind_label(wind: MahjongWind) -> &'static str {
    match wind {
        MahjongWind::East => "东",
        MahjongWind::South => "南",
        MahjongWind::West => "西",
        MahjongWind::North => "北",
    }
}
