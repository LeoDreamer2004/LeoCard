use super::hand::render_hand;
use super::{DoudizhuUiAction, DoudizhuUiState, controls, counter, summary};
use crate::app::presentation::{
    GameSummaryAnimation, MUTED, PlayEffectKind, PlayEffectState, StartGameSeatTransition,
    TableBackground, TableBackgroundMaterial, TableSeat, TableSeats, TurnBorderMaterial,
    add_auto_play_overlay, add_play_effect_overlay, add_text, table_material_params,
};
use crate::app::runtime::{AvatarImages, ClientResource, TableAppearance, UiAssets};
use crate::app::shell::{
    ChatAuxiliaryAction, ChatPanelState, SocialUiState, UiAction, add_chat_panel,
    add_reconnecting_overlay,
};
use bevy::prelude::*;
use leocard_protocol::DoudizhuSnapshot;
use leocard_ui::{UiComposer, spawn_node, ui_component};

pub(crate) struct DoudizhuTable<'a> {
    pub client: &'a ClientResource,
    pub game: &'a DoudizhuSnapshot,
    pub ui: &'a mut DoudizhuUiState,
    pub social: &'a SocialUiState,
    pub chat: &'a ChatPanelState,
    pub assets: &'a UiAssets,
    pub avatars: &'a AvatarImages,
    pub appearance: &'a TableAppearance,
    pub brightness: f32,
    pub vignette: f32,
    pub table_materials: &'a mut Assets<TableBackgroundMaterial>,
    pub turn_materials: &'a mut Assets<TurnBorderMaterial>,
    pub summary: &'a GameSummaryAnimation,
    pub transition: &'a StartGameSeatTransition,
    pub play_effect: &'a PlayEffectState,
}

pub(crate) fn render_doudizhu_table(
    commands: &mut UiComposer,
    root: Entity,
    mut context: DoudizhuTable<'_>,
) {
    let game = context.game;
    let assets = context.assets;
    let content = commands.mount(root, format!("doudizhu:{:?}", game.match_id), bsn! { @ui_component(Node {
        width: percent(100), flex_grow: 1.0, min_height: px(0), position_type: PositionType::Relative,
        flex_direction: FlexDirection::Column, ..default()
    }) });
    let material = commands.material(
        content,
        context.table_materials,
        TableBackgroundMaterial {
            params: table_material_params(
                context.brightness,
                context.vignette,
                context.appearance.custom_felt.is_none(),
            ),
            texture: context
                .appearance
                .custom_felt
                .as_ref()
                .unwrap_or(&assets.table_felt)
                .clone(),
        },
    );
    commands
        .entity(content)
        .insert((MaterialNode(material), TableBackground));
    let table = spawn_node(
        commands,
        content,
        Node {
            flex_grow: 1.0,
            width: percent(100),
            min_height: px(410),
            position_type: PositionType::Relative,
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            ..default()
        },
        None,
    );
    let hand_area = spawn_node(
        commands,
        content,
        Node {
            width: percent(100),
            height: px(210),
            flex_shrink: 0.0,
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            row_gap: px(6),
            ..default()
        },
        None,
    );
    let intro = context.transition.is_active_for(game.match_id);
    context.players(commands, table, hand_area, intro);
    if intro {
        context.ui.pending_deal = Some(game.match_id);
        return;
    }
    let status = spawn_node(
        commands,
        table,
        Node {
            position_type: PositionType::Absolute,
            top: px(10),
            left: px(18),
            ..default()
        },
        None,
    );
    add_text(
        commands,
        status,
        format!(
            "第 {}/{} 轮 第 {}/{} 局 底分 {} 倍数 ×{}",
            game.progress.round(),
            game.rules.rounds,
            game.progress.hand_in_round(),
            game.rules.mode.players(),
            game.highest_bid,
            game.multiplier
        ),
        15.0,
        MUTED,
        assets,
    );
    controls::render(commands, hand_area, game, context.ui, assets);
    let row = spawn_node(
        commands,
        hand_area,
        Node {
            height: px(146),
            justify_content: JustifyContent::Center,
            ..default()
        },
        None,
    );
    render_hand(commands, row, &game.your_hand, context.ui, assets);
    let auto = game
        .players
        .iter()
        .find(|p| p.id == game.you)
        .map(|p| p.auto_play);
    add_chat_panel(
        commands,
        content,
        context.chat,
        assets,
        game.result.is_none().then_some(auto).flatten(),
        &[ChatAuxiliaryAction {
            label: "底牌",
            action: (!game.kitty.is_empty())
                .then_some(UiAction::Doudizhu(DoudizhuUiAction::ToggleKitty)),
            highlighted: context.ui.kitty_open,
        }],
    );
    if auto == Some(true) && game.result.is_none() {
        add_auto_play_overlay(commands, content, assets);
    }
    context.kitty(commands, table);
    counter::render(commands, content, game, context.ui, assets);
    if let Some(effect) =
        context.play_effect.active.as_ref().filter(|effect| {
            effect.match_id == game.match_id && effect.kind == PlayEffectKind::Bomb
        })
    {
        let own_seat = game
            .players
            .iter()
            .find(|player| player.id == game.you)
            .map_or(0, |player| player.seat.0);
        let player_seat = game
            .players
            .iter()
            .find(|player| player.id == effect.player)
            .map_or(own_seat, |player| player.seat.0);
        let count = game.rules.mode.players();
        let relative = (player_seat + count - own_seat) % count;
        let source = match TableSeats::new(count).seat(relative) {
            TableSeat::Bottom => Vec2::new(0.0, 225.0),
            TableSeat::Top => Vec2::new(0.0, -145.0),
            TableSeat::Left(_) => Vec2::new(-385.0, 0.0),
            TableSeat::Right(_) => Vec2::new(385.0, 0.0),
        };
        add_play_effect_overlay(commands, table, content, effect.kind, source, assets);
    }
    summary::render(
        commands,
        table,
        game,
        context.avatars,
        context.summary,
        assets,
    );
    if let Some(message) = context.client.0.reconnecting_message() {
        add_reconnecting_overlay(commands, content, message, assets);
    }
}
