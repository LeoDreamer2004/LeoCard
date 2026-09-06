//! 德州扑克牌桌视图。房间、聊天、头像、桌布和按钮资源均复用公共客户端层。

use super::*;
use leocard_client::NetworkState;
use leocard_protocol::{SeatId, TABLE_SEAT_COUNT, TexasHoldemPhaseView, TexasHoldemSnapshot};

pub struct TexasTableVisuals<'a> {
    pub assets: &'a UiAssets,
    pub avatars: &'a AvatarImages,
    pub appearance: &'a TableAppearance,
    pub brightness: f32,
    pub vignette: f32,
    pub table_materials: &'a mut Assets<TableBackgroundMaterial>,
    pub turn_border_materials: &'a mut Assets<TurnBorderMaterial>,
    pub start_game_transition: &'a StartGameSeatTransition,
    pub chip_state: &'a TexasChipTableState,
    pub game_summary: &'a GameSummaryAnimation,
}

pub fn render_texas_holdem_table(
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
        start_game_transition,
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
            width: px(DESIGN_WIDTH),
            top: px(0),
            bottom: px(0),
            min_height: px(430),
            ..default()
        },
        None,
    );
    commands
        .entity(table)
        .insert(UiTransform::from_translation(Val2::px(
            -DESIGN_WIDTH * 0.5,
            0.0,
        )));
    if let NetworkState::Reconnecting(message) = client.0.state() {
        add_reconnecting_overlay(commands, content, message, assets);
    }

    let own = game
        .players
        .iter()
        .find(|player| player.id == game.you)
        .expect("德州快照必须包含接收方");
    let start_transition_active = start_game_transition.is_active_for(game.match_id);
    for relative in 1..TABLE_SEAT_COUNT {
        let physical = SeatId((own.seat.0 + relative) % TABLE_SEAT_COUNT);
        if let Some(player) = game.players.iter().find(|player| player.seat == physical) {
            add_texas_opponent(
                commands,
                table,
                player,
                relative,
                game,
                ui.social.interaction_menu_open,
                assets,
                avatars,
                turn_border_materials,
                chip_state,
                start_transition_active,
            );
        }
    }

    let new_hand = ui.texas_holdem.observed_match != Some(game.match_id)
        || ui.texas_holdem.observed_hand_number != game.hand_number;
    let new_community_from = if new_hand {
        0
    } else {
        ui.texas_holdem
            .observed_community_len
            .min(game.community.len())
    };
    let initial_deal =
        new_hand.then(|| spawn_texas_initial_deal(commands, table, game, own.seat, assets));
    let hole_card_count = client.0.model().texas_holdem_rules().map_or_else(
        || {
            if game.your_hole_cards.len() == 4 {
                4
            } else {
                2
            }
        },
        |rules| if rules.omaha { 4 } else { 2 },
    );
    add_texas_chip_areas(commands, table, game, hole_card_count, chip_state, assets);
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
        initial_deal.as_ref().map(|deal| deal.own_delays.as_slice()),
        ui,
        assets,
        avatars,
        turn_border_materials,
        chip_state,
        start_transition_active,
    );
    add_texas_showdown_reveal(commands, table, game, own.seat, assets, game_summary);
    add_texas_hand_result(commands, table, game, assets, avatars, game_summary);

    let local_auto_play =
        matches!(game.phase, TexasHoldemPhaseView::Betting { .. }).then_some(own.auto_play);
    add_chat_panel(commands, content, chat, assets, local_auto_play, None, None);
    if local_auto_play == Some(true) {
        add_auto_play_overlay(commands, content, assets);
    }

    ui.texas_holdem.observed_match = Some(game.match_id);
    ui.texas_holdem.observed_hand_number = game.hand_number;
    ui.texas_holdem.observed_community_len = game.community.len();
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
