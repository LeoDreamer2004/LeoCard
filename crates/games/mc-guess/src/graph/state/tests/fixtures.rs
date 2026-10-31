use super::super::McGuessGraphGame;
use super::super::analysis::GraphAnalysis;
use crate::graph::data::{GraphData, GraphEdge};
use crate::{McGuessGraphRelation, McGuessGraphRules};
use leocard_game_common::PlayerId;
use std::time::Duration;

pub(super) fn graph(ids: &[&str], edges: &[(usize, usize)]) -> McGuessGraphGame {
    let mut game =
        McGuessGraphGame::pending(vec![PlayerId(0), PlayerId(1)], McGuessGraphRules::default());
    let data = GraphData::bundled();
    game.nodes = ids
        .iter()
        .map(|id| {
            data.nodes
                .iter()
                .position(|node| node.entry.is_some_and(|entry| entry.id == *id))
                .unwrap()
        })
        .collect();
    game.revealed = vec![false; game.nodes.len()];
    game.analysis = GraphAnalysis::new(&game.players, game.nodes.len());
    game.edges = edges
        .iter()
        .map(|&(from, to)| GraphEdge {
            from: game.nodes[from],
            to: game.nodes[to],
            kind: McGuessGraphRelation::Other,
            directed: true,
        })
        .collect();
    game
}

pub(super) fn select(game: &mut McGuessGraphGame, number: u16) {
    game.advance_time(Duration::from_millis(600));
    game.select(game.current_player().unwrap(), number).unwrap();
}

pub(super) fn points(game: &mut McGuessGraphGame, player: PlayerId) -> u32 {
    let gains = game.take_score_gains();
    assert_eq!(gains.len(), 1);
    assert_eq!(gains[0].0, player);
    gains[0].1
}
