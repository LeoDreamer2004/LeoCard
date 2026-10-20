use bevy::prelude::*;
use leocard_minecraft_guess::{EntryKind, RoundView};
use leocard_protocol::{MatchId, MinecraftGuessSnapshot};
use std::borrow::Cow;

#[derive(Resource, Default)]
pub(crate) struct MinecraftGuessUiState {
    pub input: String,
    pub(super) graph: super::graph::GraphUiState,
    pub mode_menu_open: bool,
    pub scroll_y: f32,
    pub scroll_to_latest: bool,
    pub match_id: Option<MatchId>,
    pub game_index: u8,
    pub selected_node: Option<u16>,
    pub guess_count: usize,
}

impl MinecraftGuessUiState {
    pub(super) fn viewed_node(&self, game: &MinecraftGuessSnapshot) -> Option<u16> {
        game.graph
            .as_ref()
            .filter(|graph| !graph.nodes.is_empty())
            .map(|graph| self.graph.selected.unwrap_or(graph.selected))
    }

    pub(super) fn can_guess(&self, game: &MinecraftGuessSnapshot) -> bool {
        game.can_guess()
            && game
                .graph
                .as_ref()
                .is_none_or(|graph| self.viewed_node(game) == Some(graph.selected))
    }

    pub(super) fn can_suggest(&self, game: &MinecraftGuessSnapshot) -> bool {
        self.viewed_node(game)
            .map_or_else(|| game.can_guess(), |number| game.can_guess_node(number))
    }

    pub(super) fn viewed_round<'a>(&self, game: &'a MinecraftGuessSnapshot) -> Cow<'a, RoundView> {
        let Some(graph) = game.graph.as_ref().filter(|graph| !graph.nodes.is_empty()) else {
            return Cow::Borrowed(&game.round);
        };
        let number = self.viewed_node(game).expect("graph node");
        let node = &graph.nodes[usize::from(number - 1)];
        Cow::Owned(RoundView {
            completed: game.round.completed,
            kind: node.kind.unwrap_or(EntryKind::Block),
            character_count: node.character_count.unwrap_or(0),
            current_player: game.round.current_player,
            guesses: graph
                .guesses
                .iter()
                .filter(|record| record.node == number)
                .map(|record| record.guess.clone())
                .collect(),
            skipped_players: game.round.skipped_players.clone(),
            answer: node.entry.clone(),
            winner: None,
        })
    }
}

#[derive(Component)]
pub(super) struct GuessInput;

#[derive(Component)]
pub(super) struct GuessScroll;
