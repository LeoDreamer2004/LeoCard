use super::super::GraphGame;
use super::super::analysis::GraphAnalysis;
use crate::graph::data::{GraphData, GraphEdge};
use crate::{GraphRelation, GraphRules};
use std::time::Duration;

pub(super) fn graph(ids: &[&str], edges: &[(usize, usize)]) -> GraphGame {
    let mut game = GraphGame::pending(vec![0, 1], GraphRules::default());
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
            kind: GraphRelation::Other,
            directed: true,
        })
        .collect();
    game
}

pub(super) fn select(game: &mut GraphGame, number: u16) {
    game.advance_time(Duration::from_millis(600));
    game.select(game.current_player().unwrap(), number).unwrap();
}

pub(super) fn points(game: &mut GraphGame, player: u8) -> u32 {
    let gains = game.take_score_gains();
    assert_eq!(gains.len(), 1);
    assert_eq!(gains[0].0, player);
    gains[0].1
}
