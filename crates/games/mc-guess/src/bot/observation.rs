use crate::{McGuessEntryKind, McGuessGameState, McGuessGraphView, McGuessResult};
use leocard_game_common::PlayerId;
use std::collections::HashSet;

/// Public hints for one playable node or a regular Wordle round, without its answer.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct McGuessBotObservation {
    pub(super) node: Option<u16>,
    pub(super) kind: McGuessEntryKind,
    pub(super) character_count: u16,
    pub(super) guesses: Vec<McGuessResult>,
    pub(super) related_entries: Option<HashSet<String>>,
    pub(super) can_skip: bool,
}

impl McGuessBotObservation {
    pub fn from_game(game: &McGuessGameState, player: PlayerId) -> Option<Self> {
        let view = game.view();
        if view.finished() || view.current_player.is_some_and(|current| current != player) {
            return None;
        }
        let can_skip = view.current_player == Some(player);
        if let Some(graph) = game.graph_view() {
            Self::from_graph(&graph, can_skip)
        } else {
            Some(Self {
                node: None,
                kind: view.kind,
                character_count: view.character_count,
                guesses: view.guesses.into_iter().map(|guess| guess.result).collect(),
                related_entries: None,
                can_skip,
            })
        }
    }

    pub(super) fn from_graph(graph: &McGuessGraphView, can_skip: bool) -> Option<Self> {
        let selected = graph
            .nodes
            .iter()
            .find(|node| node.number == graph.selected && graph.can_guess_node(node.number));
        let target = selected.or_else(|| {
            graph
                .nodes
                .iter()
                .find(|node| graph.can_guess_node(node.number))
        })?;
        Some(Self {
            node: Some(target.number),
            kind: target.kind?,
            character_count: target.character_count.unwrap_or(0),
            guesses: graph
                .guesses
                .iter()
                .filter(|record| record.node == target.number)
                .map(|record| record.guess.result.clone())
                .collect(),
            related_entries: graph.candidates_from_revealed(target.number),
            can_skip,
        })
    }
}
