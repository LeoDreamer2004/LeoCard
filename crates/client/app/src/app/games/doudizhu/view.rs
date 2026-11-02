use super::hand::render_hand;
use super::{DoudizhuUiAction, DoudizhuUiState, controls, counter, summary};
use crate::app::presentation::{
    CardTableSurface, GAME_SIDE_CONTROL_SCALE, GameSummaryAnimation, MUTED, PlayEffectKind,
    PlayEffectState, StartGameSeatTransition, TableBackgroundMaterial, TableSeat, TableSeats,
    TurnBorderMaterial, add_auto_play_overlay, add_play_effect_overlay, add_text,
};
use crate::app::runtime::{AvatarImages, ClientResource, UiAssets};
use crate::app::shell::{
    ChatAuxiliaryAction, ChatPanelOptions, ChatPanelState, SocialUiState, UiAction,
    add_reconnecting_overlay,
};
use bevy::prelude::*;
use leocard_doudizhu::DoudizhuPhase;
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
    pub brightness: f32,
    pub custom_felt: Option<&'a Handle<Image>>,
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
    CardTableSurface {
        brightness: context.brightness,
        custom_felt: context.custom_felt,
    }
    .apply(commands, content, context.assets, context.table_materials);
    let table = spawn_node(
        commands,
        content,
        Node {
            flex_grow: 1.0,
            width: percent(100),
            min_height: px(if cfg!(target_os = "android") {
                330
            } else {
                410
            }),
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
            height: px(if cfg!(target_os = "android") {
                312
            } else {
                210
            }),
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
            height: px(146.0 * GAME_SIDE_CONTROL_SCALE),
            justify_content: JustifyContent::Center,
            ..default()
        },
        None,
    );
    render_hand(commands, row, &game.your_hand, context.ui, assets);
    let auto = game
        .players
        .iter()
        .find(|p| p.player.id == game.you)
        .map(|p| p.auto_play);
    ChatPanelOptions::new(
        game.result.is_none().then_some(auto).flatten(),
        &[
            ChatAuxiliaryAction {
                label: "上",
                action: (game.phase == DoudizhuPhase::Playing && context.ui.history.has_previous())
                    .then_some(UiAction::Doudizhu(DoudizhuUiAction::ShowPreviousTrick)),
                highlighted: false,
            },
            ChatAuxiliaryAction {
                label: "底",
                action: (!game.kitty.is_empty())
                    .then_some(UiAction::Doudizhu(DoudizhuUiAction::ToggleKitty)),
                highlighted: context.ui.kitty_open,
            },
        ],
    )
    .spawn(commands, content, context.chat, assets);
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
            .find(|player| player.player.id == game.you)
            .map_or(0, |player| player.seat.0);
        let player_seat = game
            .players
            .iter()
            .find(|player| player.player.id == effect.player)
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
