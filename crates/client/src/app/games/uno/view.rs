//! UNO 牌桌视图。

use super::*;
use leocard_client::NetworkState;
use leocard_protocol::{UnoPendingSwapView, UnoPhaseView, UnoSnapshot};
use leocard_uno::{UnoFlipSide, UnoPendingDrawKind, UnoRuleSet};

pub const UNO_DISCARD_OFFSETS: [(f32, f32, f32); 6] = [
    (-5.0, 4.0, -5.0),
    (4.0, 2.0, 4.0),
    (-2.0, -2.0, -2.5),
    (3.0, 1.0, 3.0),
    (-1.0, 0.0, -1.5),
    (1.0, -1.0, 2.0),
];
pub const UNO_FLYING_CARD_WIDTH: f32 = 82.0;
pub const UNO_FLYING_CARD_HEIGHT: f32 = 128.0;
pub const UNO_PALETTE_EFFECT_DURATION: f32 = 2.2;
pub const UNO_REVERSE_EFFECT_DURATION: f32 = 1.65;
pub const UNO_ACTION_AREA_BOTTOM: f32 = 153.0;
pub const UNO_ACTION_AREA_HEIGHT: f32 = 52.0;
pub struct UnoTableVisuals<'a> {
    pub assets: &'a UiAssets,
    pub avatars: &'a AvatarImages,
    pub appearance: &'a TableAppearance,
    pub brightness: f32,
    pub vignette: f32,
    pub table_materials: &'a mut Assets<TableBackgroundMaterial>,
    pub turn_border_materials: &'a mut Assets<TurnBorderMaterial>,
    pub game_summary: &'a GameSummaryAnimation,
}

pub fn render_uno_table(
    commands: &mut Commands,
    root: Entity,
    client: &ClientResource,
    game: &UnoSnapshot,
    ui: &UiState,
    chat: &ChatPanelState,
    visuals: UnoTableVisuals<'_>,
) {
    let UnoTableVisuals {
        assets,
        avatars,
        appearance,
        brightness,
        vignette,
        table_materials,
        turn_border_materials,
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
    let material = table_materials.add(TableBackgroundMaterial {
        params: table_material_params(brightness, vignette, appearance.custom_felt.is_none()),
        texture: felt,
    });
    commands
        .entity(content)
        .insert((MaterialNode(material), TableBackground));
    if game.flip_side == Some(UnoFlipSide::Dark) {
        let tint = spawn_node(
            commands,
            content,
            Node {
                position_type: PositionType::Absolute,
                width: percent(100),
                height: percent(100),
                ..default()
            },
            Some(Color::srgba(0.12, 0.07, 0.30, 0.24)),
        );
        commands.entity(tint).insert(FocusPolicy::Pass);
    }

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
        .expect("UNO 快照必须包含接收方");
    let mut opponents = game
        .players
        .iter()
        .filter(|player| player.id != game.you)
        .collect::<Vec<_>>();
    opponents.sort_by_key(|player| {
        (player.seat.0 + UnoRuleSet::MAX_PLAYERS - own.seat.0) % UnoRuleSet::MAX_PLAYERS
    });
    let opponent_count = opponents.len();
    for (index, player) in opponents.into_iter().enumerate() {
        add_uno_player_panel(
            commands,
            table,
            game,
            player,
            opponent_position(index, opponent_count),
            ui,
            avatars,
            assets,
            turn_border_materials,
        );
    }

    add_uno_center(commands, table, game, assets);
    add_uno_own_area(
        commands,
        table,
        game,
        own,
        ui,
        assets,
        turn_border_materials,
    );
    add_uno_actions(commands, table, game, ui, assets);
    add_uno_callout_actions(commands, table, game, assets);
    add_uno_swap_selection_prompt(commands, table, game, ui, assets);

    let needs_color_choice = game.current_color.is_none()
        && matches!(game.phase, UnoPhaseView::Playing)
        && (game.pending_swap.is_none()
            || matches!(
                game.pending_swap,
                Some(
                    UnoPendingSwapView::ChooseColor { .. }
                        | UnoPendingSwapView::ColorRoulette { .. }
                )
            ));
    if needs_color_choice {
        add_initial_color_choice(commands, content, game, assets);
    }
    if let Some(card) = ui.uno.color_choice {
        add_play_color_choice(commands, content, game.flip_side, card, assets);
    }
    if let UnoPhaseView::Finished {
        winner,
        results,
        reference_changes,
        ..
    } = &game.phase
    {
        add_uno_summary(
            commands,
            table,
            game,
            *winner,
            results,
            reference_changes,
            assets,
            avatars,
            game_summary,
        );
    }

    let auto_play = own.auto_play;
    add_chat_panel(commands, content, chat, assets, Some(auto_play), None, None);
    if own.eliminated && matches!(game.phase, UnoPhaseView::Playing) {
        add_uno_eliminated_own_overlay(commands, content, assets);
    } else if auto_play && matches!(game.phase, UnoPhaseView::Playing) {
        add_auto_play_overlay(commands, content, assets);
    }
}

fn add_uno_center(commands: &mut Commands, table: Entity, game: &UnoSnapshot, assets: &UiAssets) {
    let center = spawn_node(
        commands,
        table,
        Node {
            position_type: PositionType::Absolute,
            left: px(467),
            top: px(205),
            width: px(346),
            height: px(170),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            column_gap: px(34),
            ..default()
        },
        None,
    );
    let draw = spawn_node(
        commands,
        center,
        Node {
            width: px(96),
            height: px(140),
            position_type: PositionType::Relative,
            ..default()
        },
        None,
    );
    let visible_draw_cards = usize::from(game.draw_pile_len.min(6));
    for index_from_top in (0..visible_draw_cards).rev() {
        let image = game
            .draw_pile_inactive_cards
            .get(index_from_top)
            .copied()
            .map(|card| uno_card_handle(assets, card))
            .unwrap_or_else(|| assets.games.uno_card_back.clone());
        let mut card = commands.spawn((
            UnoFlipTarget::DrawPile(index_from_top),
            Node {
                position_type: PositionType::Absolute,
                left: px(7.0 - index_from_top as f32 * 1.2),
                top: px(6.0 - index_from_top as f32),
                width: px(82),
                height: px(128),
                ..default()
            },
            ImageNode::new(image),
            UiTransform::IDENTITY,
            BoxShadow::new(Color::BLACK.with_alpha(0.45), px(3), px(5), px(0), px(7)),
            FocusPolicy::Pass,
        ));
        if index_from_top == 0 {
            card.insert(UnoDrawPileAnchor);
        }
        let card = card.id();
        commands.entity(draw).add_child(card);
    }
    let draw_count = spawn_node(
        commands,
        draw,
        Node {
            position_type: PositionType::Absolute,
            right: px(-13),
            top: px(-10),
            min_width: px(31),
            height: px(25),
            padding: UiRect::horizontal(px(6)),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            border_radius: BorderRadius::all(percent(50)),
            ..default()
        },
        Some(HEADER_BG.with_alpha(0.94)),
    );
    add_text(
        commands,
        draw_count,
        game.draw_pile_len.to_string(),
        11.0,
        TEXT,
        assets,
    );

    let discard = spawn_node(
        commands,
        center,
        Node {
            width: px(96),
            height: px(140),
            position_type: PositionType::Relative,
            ..default()
        },
        None,
    );
    for (index, card) in game.discard_pile.iter().copied().enumerate() {
        let (x, y, angle) = uno_discard_pose(card);
        let mut card_entity = commands.spawn((
            UnoDiscardCard(card),
            UnoFlipTarget::DiscardPile(game.discard_pile.len() - 1 - index),
            Node {
                position_type: PositionType::Absolute,
                left: px(7.0 + x),
                top: px(6.0 + y),
                width: px(82),
                height: px(128),
                ..default()
            },
            ImageNode::new(uno_card_handle(assets, card)),
            UiTransform::from_rotation(Rot2::degrees(angle)),
            BoxShadow::new(Color::BLACK.with_alpha(0.38), px(2), px(4), px(0), px(5)),
        ));
        if index + 1 == game.discard_pile.len() {
            card_entity.insert(UnoDiscardPileAnchor);
        }
        let card = card_entity.id();
        commands.entity(discard).add_child(card);
    }

    let status = spawn_node(
        commands,
        table,
        Node {
            position_type: PositionType::Absolute,
            left: px(440),
            top: px(382),
            width: px(400),
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            row_gap: px(4),
            ..default()
        },
        None,
    );
    let current_name = game
        .current_player
        .and_then(|id| game.players.iter().find(|player| player.id == id))
        .map(|player| player.name.as_str())
        .unwrap_or("—");
    let color = game.current_color.map_or("等待选色".to_owned(), |color| {
        format!("当前颜色：{color}")
    });
    add_text(
        commands,
        status,
        format!("{color}  ·  {current_name} 的回合"),
        16.0,
        game.current_color.map_or(ACCENT, uno_ui_color),
        assets,
    );
    if game.pending_kind.is_some() {
        add_text(
            commands,
            status,
            if game.pending_kind == Some(UnoPendingDrawKind::FlipWildDrawColor) {
                format!("指定颜色摸牌 ×{}", game.pending_draw)
            } else {
                format!("累计罚牌 +{}", game.pending_draw)
            },
            17.0,
            DANGER,
            assets,
        );
    }
    if game.pending_skip > 0 {
        add_text(
            commands,
            status,
            format!("累计禁手 ×{}", game.pending_skip),
            17.0,
            DANGER,
            assets,
        );
    }
}
