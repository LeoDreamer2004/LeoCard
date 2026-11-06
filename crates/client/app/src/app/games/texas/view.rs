//! 德州扑克牌桌视图。房间、聊天、头像、桌布和按钮资源均复用公共客户端层。
use super::{
    TexasBoardReveal, TexasChipTableState, TexasShowdownSequence, TexasTableLayout, TexasUiAction,
    TexasUiState, add_second_board, add_texas_board_back, add_texas_board_face,
    add_texas_chip_areas, add_texas_hand_result, add_texas_opponent, add_texas_own_area,
    add_texas_showdown_reveal, render_texas_spectator_drawer, spawn_texas_initial_deal,
    street_label,
};
use crate::app::presentation::{
    CardTableSurface, DESIGN_WIDTH, GameSummaryAnimation, StartGameSeatTransition, TEXT,
    TableBackgroundMaterial, TableCardMaterial, TurnBorderMaterial, add_auto_play_overlay,
    add_table_card_pile, add_text,
};
use crate::app::runtime::{AvatarImages, ClientResource, UiAssets};
use crate::app::shell::{
    ChatPanelOptions, ChatPanelState, SocialUiState, UiAction, add_reconnecting_overlay,
};
use bevy::picking::Pickable;
use bevy::prelude::*;
use leocard_protocol::{GameRules, TexasPhaseView, TexasSnapshot};
use leocard_ui::{UiComposer, spawn_node, ui_component};

pub(crate) struct TexasTableVisuals<'a> {
    pub assets: &'a UiAssets,
    pub avatars: &'a AvatarImages,
    pub brightness: f32,
    pub custom_felt: Option<&'a Handle<Image>>,
    pub table_materials: &'a mut Assets<TableBackgroundMaterial>,
    pub card_materials: &'a mut Assets<TableCardMaterial>,
    pub turn_border_materials: &'a mut Assets<TurnBorderMaterial>,
    pub start_game_transition: &'a StartGameSeatTransition,
    pub chip_state: &'a TexasChipTableState,
    pub game_summary: &'a GameSummaryAnimation,
}

#[expect(
    clippy::too_many_arguments,
    reason = "the game-screen adapter passes common screen state plus grouped visuals"
)]
pub(crate) fn render_texas_table(
    commands: &mut UiComposer,
    root: Entity,
    client: &ClientResource,
    game: &TexasSnapshot,
    ui: &mut TexasUiState,
    social: &SocialUiState,
    chat: &ChatPanelState,
    visuals: TexasTableVisuals<'_>,
) {
    let TexasTableVisuals {
        assets,
        avatars,
        brightness,
        custom_felt,
        table_materials,
        card_materials,
        turn_border_materials,
        start_game_transition,
        chip_state,
        game_summary,
    } = visuals;
    let content = commands.mount(
        root,
        format!("texas:{:?}:{}", game.match_id.0, game.hand_number),
        bsn! { @ui_component(Node {
            width: percent(100),
            flex_grow: 1.0,
            min_height: px(0),
            position_type: PositionType::Relative,
            ..default()
        }) },
    );
    CardTableSurface {
        brightness,
        custom_felt,
    }
    .apply(commands, content, assets, table_materials);

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
    if let Some(message) = client.0.reconnecting_message() {
        add_reconnecting_overlay(commands, content, message, assets);
    }

    let own = game
        .players
        .iter()
        .find(|player| player.player.id == game.you);
    let layout = TexasTableLayout::for_game(game);
    let start_transition_active = start_game_transition.is_active_for(game.match_id);
    for player in game
        .players
        .iter()
        .filter(|player| player.player.id != game.you)
    {
        let relative = layout.player_slot(game, player.player.id);
        add_texas_opponent(
            commands,
            table,
            player,
            relative,
            game,
            social.interaction_menu_open,
            assets,
            avatars,
            turn_border_materials,
            chip_state,
            start_transition_active,
            start_transition_active,
            &ui.spectator,
        );
    }

    if start_transition_active && let Some(own) = own {
        add_texas_own_area(
            commands,
            table,
            game,
            own,
            None,
            ui,
            social.interaction_menu_open,
            assets,
            avatars,
            turn_border_materials,
            chip_state,
            true,
            true,
        );
        return;
    }

    let sequence = TexasShowdownSequence::new(game, chip_state.runout_reveal_duration);
    let new_hand = ui.observed_table.observe((game.match_id, game.hand_number));
    let new_community_from = if new_hand {
        0
    } else {
        ui.observed_table.state.min(game.community.len())
    };
    let initial_deal = new_hand.then(|| spawn_texas_initial_deal(commands, table, game, assets));
    let hole_card_count = client
        .0
        .model()
        .game_rules()
        .and_then(GameRules::texas)
        .map_or_else(
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
        TexasCommunityAnimation {
            reveal: TexasBoardReveal {
                first_new: new_community_from,
                batch: chip_state.runout_reveal_duration > 0.0,
            },
            initial_deal_duration: initial_deal.as_ref().map(|deal| deal.duration),
            sequence,
        },
        assets,
        card_materials,
    );
    if let Some(own) = own {
        add_texas_own_area(
            commands,
            table,
            game,
            own,
            initial_deal.as_ref(),
            ui,
            social.interaction_menu_open,
            assets,
            avatars,
            turn_border_materials,
            chip_state,
            start_transition_active,
            false,
        );
    }
    render_texas_spectator_drawer(commands, content, game, ui, assets);
    add_texas_showdown_reveal(commands, table, game, assets, game_summary, sequence);
    add_texas_hand_result(commands, table, game, assets, avatars, game_summary);

    let local_auto_play = matches!(
        game.phase,
        TexasPhaseView::Betting { .. } | TexasPhaseView::ChoosingRunout
    )
    .then(|| own.map(|player| player.auto_play))
    .flatten();
    ChatPanelOptions {
        help: Some(UiAction::Texas(TexasUiAction::ToggleHandGuide)),
        ..ChatPanelOptions::new(local_auto_play, &[])
    }
    .spawn(commands, content, chat, assets);
    if local_auto_play == Some(true) {
        add_auto_play_overlay(commands, content, assets);
    }

    ui.observed_table.state = game.community.len();
}

struct TexasCommunityAnimation {
    reveal: TexasBoardReveal,
    initial_deal_duration: Option<f32>,
    sequence: TexasShowdownSequence,
}

fn add_community_area(
    commands: &mut UiComposer,
    table: Entity,
    game: &TexasSnapshot,
    animation: TexasCommunityAnimation,
    assets: &UiAssets,
    card_materials: &mut Assets<TableCardMaterial>,
) {
    let TexasCommunityAnimation {
        reveal,
        initial_deal_duration,
        sequence,
    } = animation;
    if !game.second_community.is_empty() {
        add_second_board(
            commands,
            table,
            game,
            reveal,
            sequence,
            assets,
            card_materials,
        );
        return;
    }
    let center = spawn_node(
        commands,
        table,
        Node {
            position_type: PositionType::Absolute,
            left: percent(50),
            top: px(TexasTableLayout::BOARD_TOP),
            width: px(TexasTableLayout::board_width()),
            height: px(TexasTableLayout::BOARD_ROW_HEIGHT),
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            ..default()
        },
        None,
    );
    commands.entity(center).insert((
        UiTransform::from_translation(Val2::px(-TexasTableLayout::board_width() * 0.5, 0.0)),
        // 公共牌始终绘制在筹码区背景框之上。
        ZIndex(10),
    ));
    let cards = spawn_node(
        commands,
        center,
        Node {
            width: percent(100),
            height: px(TexasTableLayout::BOARD_ROW_HEIGHT),
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
            width: px(54),
            height: px(TexasTableLayout::BOARD_ROW_HEIGHT),
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            row_gap: px(2),
            flex_shrink: 0.0,
            ..default()
        },
        None,
    );
    let pile = add_table_card_pile(
        commands,
        pile_column,
        visual_draw_pile,
        assets,
        card_materials,
    );
    commands
        .entity(pile)
        .entry::<Node>()
        .and_modify(|mut node| {
            node.position_type = PositionType::Absolute;
            node.left = px(2);
            node.top = px(8);
        });
    let label = add_text(
        commands,
        pile_column,
        street_label(&game.phase),
        12.0,
        TEXT,
        assets,
    );
    commands
        .entity(label)
        .entry::<Node>()
        .and_modify(|mut node| {
            node.position_type = PositionType::Absolute;
            node.left = px(0);
            node.top = px(80);
            node.width = percent(100);
        });
    commands
        .entity(label)
        .insert(TextLayout::justify(Justify::Center));
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
    commands.entity(gap).insert(Pickable::IGNORE);
    for index in 0..5 {
        if let Some(card) = game.community.get(index).copied() {
            add_texas_board_face(
                commands,
                cards,
                card,
                reveal.flip_delay(index),
                assets,
                card_materials,
            );
        } else {
            add_texas_board_back(
                commands,
                cards,
                index,
                initial_deal_duration.map(|duration| duration + index as f32 * 0.065),
                assets,
                card_materials,
            );
        }
    }
}
