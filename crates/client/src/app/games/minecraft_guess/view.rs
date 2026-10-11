use super::{GuessScorePlayback, MinecraftGuessAssets, state::MinecraftGuessUiState};
use crate::app::{
    presentation::{
        ACCENT, GameSummaryAnimation, MUTED, StartGameSeatTransition, TEXT, TurnBorderMaterial,
        add_text, spawn_node,
    },
    runtime::{AvatarImages, ClientResource, UiAssets},
    shell::{ChatPanelState, SocialUiState, add_chat_panel, add_reconnecting_overlay},
};
use bevy::prelude::*;
use leocard_client::NetworkState;
use leocard_protocol::MinecraftGuessSnapshot;

pub(crate) struct GuessTable<'a> {
    pub client: &'a ClientResource,
    pub game: &'a MinecraftGuessSnapshot,
    pub state: &'a MinecraftGuessUiState,
    pub scores: &'a GuessScorePlayback,
    pub summary: &'a GameSummaryAnimation,
    pub social: &'a SocialUiState,
    pub chat: &'a ChatPanelState,
    pub assets: &'a UiAssets,
    pub game_assets: &'a MinecraftGuessAssets,
    pub avatars: &'a AvatarImages,
    pub transition: &'a StartGameSeatTransition,
    pub turn_border_materials: &'a mut Assets<TurnBorderMaterial>,
}

pub(crate) fn render_guess_table(commands: &mut Commands, root: Entity, mut table: GuessTable<'_>) {
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
                top: px(12),
                bottom: px(12),
            },
            flex_direction: FlexDirection::Column,
            row_gap: px(12),
            position_type: PositionType::Relative,
            ..default()
        },
        None,
    );
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
    table.answer(commands, heading);
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
    table.controls(commands, page);
    table.players(commands, page);
    table.summary(commands, page);
    add_chat_panel(commands, page, table.chat, table.assets, None, &[]);
    if let NetworkState::Reconnecting(message) = table.client.0.state() {
        add_reconnecting_overlay(commands, page, message, table.assets);
    }
}

impl GuessTable<'_> {
    fn answer(&self, commands: &mut Commands, heading: Entity) {
        let progress = self.game.progress;
        let label = add_text(
            commands,
            heading,
            format!(
                "第 {}/{} 轮  第 {}/{} 局",
                progress.current_round(),
                progress.rounds,
                progress.game_index + 1,
                progress.total_games(),
            ),
            14.0,
            MUTED,
            self.assets,
        );
        commands.entity(label).insert(Node {
            position_type: PositionType::Absolute,
            left: px(0),
            ..default()
        });
        let (value, color) = match &self.game.round.answer {
            Some(answer) => (answer.name.clone(), ACCENT),
            None => (
                "_ ".repeat(usize::from(self.game.round.character_count))
                    .trim_end()
                    .to_owned(),
                TEXT,
            ),
        };
        let label = add_text(
            commands,
            heading,
            format!("{}：{value}", self.game.round.kind.label()),
            23.0,
            color,
            self.assets,
        );
        commands.entity(label).insert((
            TextLayout::justify(Justify::Center),
            Node {
                max_width: percent(100),
                ..default()
            },
        ));
    }
}
