use super::actions::GuandanUiAction;
use super::controls;
use super::counter;
use super::hand::render_hand;
use super::state::GuandanUiState;
use super::summary;
use super::tribute;
use crate::app::presentation::{
    ACCENT, CardTableSurface, GAME_SIDE_CONTROL_SCALE, GameSummaryAnimation, PlayEffectState,
    StartGameSeatTransition, TEXT, TableBackgroundMaterial, TableSeat, TableSeats,
    TurnBorderMaterial, add_auto_play_overlay, add_hanging_table_sign, add_play_effect_overlay,
    add_text,
};
use crate::app::runtime::{AvatarImages, ClientResource, UiAssets};
use crate::app::shell::{
    ChatAuxiliaryAction, ChatPanelOptions, ChatPanelState, SocialUiState, UiAction,
    add_reconnecting_overlay,
};
use bevy::prelude::*;
use leocard_guandan::{GuandanPhase, rank_label};
use leocard_protocol::GuandanSnapshot;
use leocard_ui::{UiComposer, spawn_node, ui_component};
use std::mem;

pub(super) struct GuandanTable<'a> {
    pub client: &'a ClientResource,
    pub game: &'a GuandanSnapshot,
    pub ui: &'a mut GuandanUiState,
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

pub(super) fn render_table(commands: &mut UiComposer, root: Entity, mut context: GuandanTable<'_>) {
    let game = context.game;
    let assets = context.assets;
    let can_control = context.client.0.model().you() == Some(game.you);
    let content = commands.mount(root, format!("guandan:{:?}", game.match_id), bsn! { @ui_component(Node {
        width: percent(100), flex_grow: 1.0, min_height: px(0), position_type: PositionType::Relative,
        flex_direction: FlexDirection::Column, ..default()
    }) });
    CardTableSurface {
        brightness: context.brightness,
        custom_felt: context.custom_felt,
    }
    .apply(commands, content, assets, context.table_materials);
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
    let portraits = context.players(commands, table, hand_area, intro);
    if intro {
        context.ui.pending_deal = true;
        return;
    }
    tribute::add_flights(
        commands,
        portraits,
        mem::take(&mut context.ui.tribute_flights),
        assets,
    );
    let sign = add_hanging_table_sign(
        commands,
        table,
        assets,
        GAME_SIDE_CONTROL_SCALE,
        false,
        326.0,
    );
    let status = sign.content;
    commands
        .entity(status)
        .entry::<Node>()
        .and_modify(|mut node| {
            node.flex_direction = FlexDirection::Column;
            node.justify_content = JustifyContent::Center;
            node.row_gap = px(3);
        });
    add_text(
        commands,
        status,
        format!(
            "第 {} 局 当前打 {}",
            game.hand_number,
            rank_label(game.level)
        ),
        17.0,
        TEXT,
        assets,
    );
    add_text(
        commands,
        status,
        format!(
            "一队 {}  二队 {}",
            rank_label(game.team_levels[0]),
            rank_label(game.team_levels[1])
        ),
        19.0,
        ACCENT,
        assets,
    );
    if game.tribute_resisted {
        add_text(commands, status, "双大王抗贡", 14.0, TEXT, assets);
    } else if matches!(game.phase, GuandanPhase::Tribute) {
        if let Some(route) = tribute::status(game) {
            add_text(commands, status, route, 14.0, TEXT, assets);
        }
    } else if context.ui.previous_reveal > 0.0 {
        add_text(commands, status, "上一圈回看", 14.0, ACCENT, assets);
    }
    controls::render(commands, hand_area, game, context.ui, can_control, assets);
    let hand_row = spawn_node(
        commands,
        hand_area,
        Node {
            height: px(146.0 * GAME_SIDE_CONTROL_SCALE),
            justify_content: JustifyContent::Center,
            ..default()
        },
        None,
    );
    render_hand(commands, hand_row, game, context.ui, can_control, assets);
    let auto = game
        .players
        .iter()
        .find(|player| player.player.id == game.you)
        .map(|player| player.auto_play);
    ChatPanelOptions::new(
        game.result.is_none().then_some(auto).flatten(),
        &[ChatAuxiliaryAction {
            label: "上",
            action: (matches!(game.phase, GuandanPhase::Playing) && game.previous_trick.is_some())
                .then_some(UiAction::Guandan(GuandanUiAction::ShowPreviousTrick)),
            highlighted: context.ui.previous_reveal > 0.0,
        }],
    )
    .spawn(commands, content, context.chat, assets);
    if can_control && auto == Some(true) && game.result.is_none() {
        add_auto_play_overlay(commands, content, assets);
    }
    counter::render(commands, content, game, context.ui, assets);
    if let Some(effect) = context
        .play_effect
        .active
        .as_ref()
        .filter(|effect| effect.match_id == game.match_id && effect.kind.is_bomb())
    {
        let seat_of = |id| {
            game.players
                .iter()
                .find(|player| player.player.id == id)
                .map_or(0, |player| player.seat.0)
        };
        let seats = TableSeats::new(4);
        let source = match seats.seat(seats.relative(seat_of(game.you), seat_of(effect.player))) {
            TableSeat::Bottom => Vec2::new(0.0, 180.0),
            TableSeat::Top => Vec2::new(0.0, -100.0),
            TableSeat::Left(_) => Vec2::new(-322.0, 18.0),
            TableSeat::Right(_) => Vec2::new(322.0, 18.0),
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
