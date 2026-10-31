use super::background::add_guess_background;
use super::state::McGuessUiState;
use super::{GuessFeedback, GuessScorePlayback, McGuessAssets, McGuessUiAction};
use crate::app::presentation::{GameSummaryAnimation, StartGameSeatTransition, TurnBorderMaterial};
use crate::app::runtime::{AvatarImages, ClientResource, UiAssets};
use crate::app::shell::{
    ChatPanelState, SocialUiState, UiAction, add_chat_help_button, add_chat_panel,
    add_reconnecting_overlay,
};
use bevy::prelude::*;
use leocard_protocol::McGuessSnapshot;
use leocard_ui::{UiComposer, spawn_node};

pub(crate) struct GuessTable<'a> {
    pub client: &'a ClientResource,
    pub game: &'a McGuessSnapshot,
    pub state: &'a McGuessUiState,
    pub graph_art: &'a super::GraphArt,
    pub scores: &'a GuessScorePlayback,
    pub feedback: &'a GuessFeedback,
    pub summary: &'a GameSummaryAnimation,
    pub social: &'a SocialUiState,
    pub chat: &'a ChatPanelState,
    pub assets: &'a UiAssets,
    pub game_assets: &'a McGuessAssets,
    pub avatars: &'a AvatarImages,
    pub transition: &'a StartGameSeatTransition,
    pub turn_border_materials: &'a mut Assets<TurnBorderMaterial>,
}

pub(crate) fn render_guess_table(
    commands: &mut UiComposer,
    root: Entity,
    mut table: GuessTable<'_>,
) {
    add_guess_background(commands, root);
    let page = spawn_node(
        commands,
        root,
        Node {
            width: percent(100),
            flex_grow: 1.0,
            min_height: px(0),
            padding: UiRect {
                left: px(24),
                right: px(24),
                top: px(if table.game.graph.is_some() {
                    6.0
                } else {
                    12.0
                }),
                bottom: px(12),
            },
            flex_direction: FlexDirection::Column,
            row_gap: px(if table.game.graph.is_some() {
                8.0
            } else {
                12.0
            }),
            position_type: PositionType::Relative,
            ..default()
        },
        None,
    );
    if table.game.graph.is_some() {
        table.graph_workspace(commands, page);
    } else {
        let heading = spawn_node(
            commands,
            page,
            Node {
                width: percent(100),
                min_height: px(38),
                flex_shrink: 0.0,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                column_gap: px(10),
                padding: UiRect::horizontal(px(180)),
                ..default()
            },
            None,
        );
        if table.transition.is_active_for(table.game.match_id) {
            commands.entity(heading).insert(Visibility::Hidden);
        }
        table.answer(commands, heading, page);
        let panel = spawn_node(
            commands,
            page,
            Node {
                width: percent(100),
                height: px(0),
                flex_grow: 1.0,
                flex_basis: px(0),
                min_height: px(0),
                padding: UiRect::all(px(10)),
                flex_direction: FlexDirection::Column,
                ..default()
            },
            None,
        );
        commands.entity(panel).insert(table.game_assets.frame());
        if table.transition.is_active_for(table.game.match_id) {
            commands.entity(panel).insert(Visibility::Hidden);
        }
        table.attributes(commands, panel);
    }
    if table.game.graph.is_none() {
        table.controls(commands, page);
        table.players(commands, page);
    }
    table.summary(commands, page);
    let chat_panel = add_chat_panel(commands, page, table.chat, table.assets, None, &[]);
    if table.game.graph.is_some() {
        add_chat_help_button(
            commands,
            chat_panel,
            UiAction::McGuess(McGuessUiAction::ToggleGraphGuide),
            table.assets,
        );
    }
    if let Some(message) = table.client.0.reconnecting_message() {
        add_reconnecting_overlay(commands, page, message, table.assets);
    }
}
