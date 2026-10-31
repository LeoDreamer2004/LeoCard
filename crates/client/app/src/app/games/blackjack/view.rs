use super::{BlackjackPlayback, BlackjackUiState};
use crate::app::presentation::{
    DESIGN_WIDTH, GameSummaryAnimation, StartGameSeatTransition, TableBackground,
    TableBackgroundMaterial, TurnBorderMaterial, add_auto_play_overlay, table_material_params,
};
use crate::app::runtime::{AvatarImages, ClientResource, TableAppearance, UiAssets};
use crate::app::shell::{
    ChatPanelState, SocialUiState, UiAction, add_chat_help_button, add_chat_panel,
    add_reconnecting_overlay,
};
use leocard_blackjack::BlackjackPhase;
use leocard_protocol::BlackjackSnapshot;
use leocard_ui::{UiComposer, spawn_node, ui_component};

use bevy::prelude::*;

pub(crate) struct BlackjackTable<'a> {
    pub client: &'a ClientResource,
    pub game: &'a BlackjackSnapshot,
    pub ui: &'a BlackjackUiState,
    pub playback: &'a BlackjackPlayback,
    pub social: &'a SocialUiState,
    pub chat: &'a ChatPanelState,
    pub assets: &'a UiAssets,
    pub avatars: &'a AvatarImages,
    pub appearance: &'a TableAppearance,
    pub brightness: f32,
    pub vignette: f32,
    pub table_materials: &'a mut Assets<TableBackgroundMaterial>,
    pub turn_materials: &'a mut Assets<TurnBorderMaterial>,
    pub transition: &'a StartGameSeatTransition,
    pub summary: &'a GameSummaryAnimation,
}

pub(crate) fn render_blackjack_table(
    commands: &mut UiComposer,
    root: Entity,
    mut context: BlackjackTable<'_>,
) {
    let game = context.game;
    let content = commands.mount(root, format!("blackjack:{:?}:{}", game.match_id, game.round), bsn! { @ui_component(Node {
        width: percent(100), flex_grow: 1.0, min_height: px(0), position_type: PositionType::Relative, ..default()
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
                .unwrap_or(&context.assets.table_felt)
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
            position_type: PositionType::Absolute,
            left: percent(50),
            top: px(0),
            bottom: px(0),
            width: px(DESIGN_WIDTH),
            min_height: px(660),
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
    let pile = if context.transition.is_active_for(game.match_id) {
        table
    } else {
        context.dealer(commands, table)
    };
    context.players(commands, table, pile);
    if !context.transition.is_active_for(game.match_id) {
        context.controls(commands, table);
        context.summary(commands, table);
    }
    let own = game
        .players
        .iter()
        .find(|player| player.player.id == game.you);
    let auto_play = (game.phase != BlackjackPhase::Complete)
        .then(|| own.map(|player| player.auto_play))
        .flatten();
    let chat = add_chat_panel(
        commands,
        content,
        context.chat,
        context.assets,
        auto_play,
        &[],
    );
    add_chat_help_button(
        commands,
        chat,
        UiAction::Blackjack(super::BlackjackUiAction::ToggleGuide),
        context.assets,
    );
    if auto_play == Some(true) {
        add_auto_play_overlay(commands, content, context.assets);
    }
    if context.ui.guide_open {
        context.guide(commands, content);
    }
    if let Some(message) = context.client.0.reconnecting_message() {
        add_reconnecting_overlay(commands, content, message, context.assets);
    }
}
