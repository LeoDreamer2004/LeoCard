use super::super::GraphGame;
use super::super::analysis::GraphAnalysis;
use super::fixtures::{graph, select};
use crate::graph::data::{GraphData, GraphEdge};
use crate::{GameError, GraphRelation, GuessActionStatistics};

fn action(game: &mut GraphGame, player: u8) -> GuessActionStatistics {
    game.take_action_statistics()
        .into_iter()
        .rev()
        .find(|action| action.player == player && action.graph.is_some())
        .unwrap()
}

#[test]
fn combos_openings_and_sweeps_require_real_consecutive_answers() {
    let ids = [
        "minecraft:stone",
        "minecraft:dirt",
        "minecraft:diamond",
        "minecraft:stick",
        "minecraft:emerald",
    ];
    for skip_first in [false, true] {
        let mut game = graph(&ids, &[(0, 1), (1, 2), (2, 3), (3, 4)]);
        if skip_first {
            game.skip(0).unwrap();
            game.take_action_statistics();
        }
        let player = game.current_player().unwrap();
        for (index, id) in ids.iter().enumerate() {
            select(&mut game, index as u16 + 1);
            game.guess(player, id).unwrap();
            let facts = action(&mut game, player).graph.unwrap();
            assert_eq!(facts.combo, index as u16 + 1);
            assert_eq!(facts.opening_win, index == 0 && !skip_first);
            assert_eq!(facts.combo_sweep, index == ids.len() - 1);
        }
        assert!(game.finished());
    }
    let mut game = graph(&ids, &[(0, 1), (1, 2), (2, 3), (3, 4)]);
    game.guess(0, ids[0]).unwrap();
    game.skip(0).unwrap();
    game.take_action_statistics();
    for (index, id) in ids.iter().enumerate().skip(1) {
        select(&mut game, index as u16 + 1);
        game.guess(1, id).unwrap();
        assert!(!action(&mut game, 1).graph.unwrap().combo_sweep);
    }
}

#[test]
fn graph_first_greens_and_assists_count_distinct_nodes_in_one_graph() {
    let ids = [
        "minecraft:blue_concrete",
        "minecraft:purple_concrete",
        "minecraft:cyan_concrete",
        "minecraft:red_concrete",
        "minecraft:green_concrete",
    ];
    for assist in [false, true] {
        let mut game = graph(&ids, &[(0, 1), (1, 2), (2, 3), (3, 4)]);
        for (index, id) in ids.iter().enumerate() {
            select(&mut game, index as u16 + 1);
            game.guess(0, "minecraft:white_concrete").unwrap();
            let facts = action(&mut game, 0).graph.unwrap();
            assert_eq!(facts.first_green_nodes, index as u32 + 1);
            assert_eq!(
                game.guess(1, "minecraft:white_concrete"),
                Err(GameError::AlreadyGuessed)
            );
            if assist {
                game.guess(1, id).unwrap();
                let facts = action(&mut game, 1).graph.unwrap();
                assert_eq!(facts.assisted_nodes, index as u32 + 1);
                assert_eq!(facts.first_green_nodes, 0);
            }
            if !game.finished() {
                game.skip(1).unwrap();
                game.take_action_statistics();
            }
        }
    }
    let mut fresh = graph(&ids, &[(0, 1), (1, 2), (2, 3), (3, 4)]);
    fresh.guess(0, "minecraft:white_concrete").unwrap();
    assert_eq!(action(&mut fresh, 0).graph.unwrap().first_green_nodes, 1);
}

#[test]
fn tag_only_neighbours_and_length_hints_use_the_selected_node() {
    let mut game = graph(&["minecraft:stone", "minecraft:stick"], &[(0, 1)]);
    let data = GraphData::bundled();
    let tag = data
        .nodes
        .iter()
        .position(|node| node.entry.is_none())
        .unwrap();
    game.nodes.push(tag);
    game.revealed.push(false);
    game.analysis = GraphAnalysis::new(&game.players, 3);
    game.edges = [0, 1]
        .into_iter()
        .map(|index| GraphEdge {
            from: game.nodes[index],
            to: tag,
            kind: GraphRelation::Other,
            directed: true,
        })
        .collect();
    game.guess(0, "minecraft:stone").unwrap();
    assert!(action(&mut game, 0).graph.unwrap().only_tag_neighbours);
    select(&mut game, 2);
    game.guess(0, "minecraft:stick").unwrap();
    assert!(action(&mut game, 0).graph.unwrap().only_tag_neighbours);
    assert!(game.finished());
    for show in [false, true] {
        let mut game = graph(&["minecraft:blue_concrete", "minecraft:stick"], &[(0, 1)]);
        game.rules.show_character_count = show;
        game.guess(0, "minecraft:stone").unwrap();
        assert_eq!(action(&mut game, 0).wrong_length, show);
    }
}

#[test]
fn round_wins_use_this_graphs_scores_allow_ties_and_ignore_developer_reveals() {
    let mut game = graph(&["minecraft:stone", "minecraft:stick"], &[(0, 1)]);
    game.skip(0).unwrap();
    game.skip(1).unwrap();
    let winners: Vec<_> = game
        .take_action_statistics()
        .into_iter()
        .filter(|action| action.graph.is_some_and(|graph| graph.round_win))
        .map(|action| action.player)
        .collect();
    assert_eq!(winners, vec![0, 1]);
    assert!(game.take_action_statistics().is_empty());
    let mut game = graph(&["minecraft:stone", "minecraft:stick"], &[(0, 1)]);
    game.guess(0, "minecraft:dirt").unwrap();
    game.take_action_statistics();
    game.skip(1).unwrap();
    game.skip(0).unwrap();
    let winners: Vec<_> = game
        .take_action_statistics()
        .into_iter()
        .filter(|action| action.graph.is_some_and(|graph| graph.round_win))
        .map(|action| action.player)
        .collect();
    assert_eq!(winners, vec![0]);
    let mut developer = graph(&["minecraft:stone", "minecraft:stick"], &[(0, 1)]);
    developer.reveal_all().unwrap();
    assert!(developer.take_action_statistics().is_empty());
}
