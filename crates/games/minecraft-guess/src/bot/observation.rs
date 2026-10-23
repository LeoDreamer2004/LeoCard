use crate::{EntryKind, GameState, GuessResult};

/// Public hints for one playable node or a regular Wordle round, without its answer.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MinecraftGuessBotObservation {
    pub(super) node: Option<u16>,
    pub(super) kind: EntryKind,
    pub(super) character_count: u16,
    pub(super) guesses: Vec<GuessResult>,
    pub(super) can_skip: bool,
}

impl MinecraftGuessBotObservation {
    pub fn from_game(game: &GameState, player: u8) -> Option<Self> {
        let view = game.view();
        if view.finished() || view.current_player.is_some_and(|current| current != player) {
            return None;
        }
        let can_skip = view.current_player == Some(player);
        if let Some(graph) = game.graph_view() {
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
                can_skip,
            })
        } else {
            Some(Self {
                node: None,
                kind: view.kind,
                character_count: view.character_count,
                guesses: view.guesses.into_iter().map(|guess| guess.result).collect(),
                can_skip,
            })
        }
    }
}
