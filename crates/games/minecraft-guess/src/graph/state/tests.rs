use self::fixtures::points;
use super::GraphGame;
use crate::graph::data::GraphData;
use crate::{GameError, GraphNodeView, GraphRules, GuessResult, MinecraftGuessRuleSet};
use std::time::Duration;

mod analysis;
mod combo;
mod fixtures;
mod scoring;
mod statistics;

#[test]
fn revealing_all_also_unlocks_tags_without_inventing_guesses_or_points() {
    let mut pending = GraphGame::pending(vec![0, 1], GraphRules::default());
    assert_eq!(pending.reveal_all(), Err(GameError::GraphUnavailable));
    assert!(!pending.finished());
    let mut game = GraphGame::new(vec![0, 1], GraphRules::default(), 5).unwrap();
    game.reveal_all().unwrap();
    assert!(game.finished());
    assert!(game.view().nodes.iter().all(GraphNodeView::revealed));
    assert!(game.view().edges.iter().all(|edge| edge.active));
    assert!(game.view().guesses.is_empty());
    assert!(game.take_score_gains().is_empty());
    assert_eq!(game.current_player(), None);
    assert_eq!(game.reveal_all(), Err(GameError::Finished));
}

#[test]
fn developer_pair_is_gated_and_completes_after_two_real_answers() {
    let enabled = cfg!(debug_assertions) || cfg!(feature = "developer");
    assert_eq!(GraphRules::NODE_OPTIONS.contains(&2), enabled);
    for include_tags in [false, true] {
        let rules = GraphRules {
            nodes: 2,
            include_tags,
            ..Default::default()
        };
        assert_eq!(
            MinecraftGuessRuleSet {
                graph: rules,
                ..Default::default()
            }
            .is_valid(),
            enabled
        );
        if !enabled {
            assert!(matches!(
                GraphGame::new(vec![0, 1], rules, 27),
                Err(GameError::GraphUnavailable)
            ));
            continue;
        }
        let mut game = GraphGame::new(vec![0, 1], rules, 27).unwrap();
        assert_eq!(
            game.view(),
            GraphGame::new(vec![0, 1], rules, 27).unwrap().view()
        );
        assert_eq!(game.nodes.len(), 2);
        assert_eq!(game.edges.len(), 1);
        assert_ne!(game.nodes[0], game.nodes[1]);
        assert!(
            game.view()
                .nodes
                .iter()
                .all(|node| node.kind.is_some() && !node.revealed())
        );
        let edge = game.edges[0];
        assert!(GraphData::bundled().edges.iter().any(|real| {
            real.from == edge.from && real.to == edge.to && real.kind == edge.kind
        }));
        for index in 0..2 {
            let answer = game.entry(index).unwrap().id.clone();
            let player = game.current_player().unwrap();
            game.select(player, index as u16 + 1).unwrap();
            game.guess(player, &answer).unwrap();
            assert_eq!(game.finished(), index == 1);
            assert!(points(&mut game, player) > 0);
        }
        assert!(game.view().edges[0].active);
    }
}

#[test]
fn meshes_hide_answers_and_never_leave_tags_waiting_on_tags() {
    for nodes in [16, 24, 32] {
        for include_tags in [false, true] {
            let game = GraphGame::new(
                vec![0, 1],
                GraphRules {
                    nodes,
                    include_tags,
                    ..GraphRules::default()
                },
                27,
            )
            .unwrap();
            let view = game.view();
            let entries: Vec<_> = view
                .nodes
                .iter()
                .filter(|node| node.kind.is_some())
                .collect();
            assert_eq!(entries.len(), usize::from(nodes));
            assert!(
                entries
                    .iter()
                    .enumerate()
                    .all(|(index, node)| { usize::from(node.number) == index + 1 })
            );
            let tags = view.nodes.len() - entries.len();
            assert!(
                tags <= if include_tags {
                    usize::from(nodes / 8).max(2)
                } else {
                    0
                }
            );
            assert_eq!(view.selected, 1);
            assert!(
                view.nodes
                    .iter()
                    .all(|node| node.entry.is_none() && node.tag_name.is_none())
            );
            assert!(
                view.edges
                    .iter()
                    .all(|edge| !edge.active && edge.directed.is_none())
            );
            for node in &view.nodes {
                let neighbours: Vec<_> = view
                    .edges
                    .iter()
                    .filter_map(|edge| {
                        if edge.source == node.number {
                            Some(edge.target)
                        } else if edge.target == node.number {
                            Some(edge.source)
                        } else {
                            None
                        }
                    })
                    .collect();
                assert!(!neighbours.is_empty());
                if node.kind.is_none() {
                    assert!(include_tags);
                    assert!(neighbours.len() >= 2);
                    assert!(
                        neighbours
                            .into_iter()
                            .all(|id| view.nodes[usize::from(id - 1)].kind.is_some())
                    );
                }
            }
        }
    }
}

#[test]
fn turns_node_histories_cooldown_scoring_and_tag_completion() {
    let mut game = GraphGame::new(vec![0, 1], GraphRules::default(), 5).unwrap();
    assert_eq!(game.select(1, 2), Err(GameError::NotPlayersTurn));
    game.select(0, 2).unwrap();
    assert_eq!(game.select(0, 3), Err(GameError::NodeSwitchCoolingDown));
    game.advance_time(Duration::from_millis(600));
    game.select(0, 1).unwrap();
    let answer = game.entry(0).unwrap().clone();
    let wrong = crate::Catalogue::bundled()
        .search(answer.kind, "")
        .find(|entry| {
            entry.id != answer.id
                && !GuessResult::resolve(&answer, &entry.id, false)
                    .unwrap()
                    .all_matching()
        })
        .unwrap();
    game.guess(0, &wrong.id).unwrap();
    assert_eq!(game.current_player(), Some(1));
    assert!((50..250).contains(&points(&mut game, 0)));
    assert_eq!(game.round_view().guesses.len(), 1);
    game.guess(1, &answer.id).unwrap();
    assert!(points(&mut game, 1) > 0);
    assert!(!game.finished());
    while !game.finished() {
        let index = game.available_nodes()[0];
        let answer = game.entry(index).unwrap().clone();
        let player = game.current_player().unwrap();
        game.advance_time(Duration::from_millis(600));
        game.select(player, index as u16 + 1).unwrap();
        game.guess(player, &answer.id).unwrap();
        assert!(points(&mut game, player) > 0);
    }
    assert!(game.finished());
    assert!(game.view().nodes.iter().all(GraphNodeView::revealed));
    assert!(
        game.view()
            .edges
            .iter()
            .all(|edge| edge.active && edge.directed.is_some())
    );
    assert_eq!(
        game.view()
            .guesses
            .iter()
            .filter(|record| record.node == 1)
            .count(),
        2
    );
}

#[test]
fn public_history_keeps_guesses_in_order_across_node_switches_without_revealing_answers() {
    let mut rules = GraphRules::default();
    rules.options.candidate_hints = false;
    rules.options.allow_unknown_guesses = true;
    let mut game = GraphGame::new(vec![0, 1], rules, 5).unwrap();
    for (number, text) in [(1, "未知甲"), (2, "未知乙"), (1, "未知丙")] {
        game.advance_time(Duration::from_millis(600));
        let player = game.current_player().unwrap();
        game.select(player, number).unwrap();
        game.guess(player, text).unwrap();
    }
    let view = game.view();
    assert_eq!(
        view.guesses
            .iter()
            .map(|record| (record.node, record.guess.player, record.guess.result.name()))
            .collect::<Vec<_>>(),
        [(1, 0, "未知甲"), (2, 1, "未知乙"), (1, 0, "未知丙")]
    );
    assert!(view.nodes.iter().all(|node| !node.revealed()));
    assert_eq!(game.round_view().guesses.len(), 2);
    game.advance_time(Duration::from_millis(600));
    game.select(1, 2).unwrap();
    assert_eq!(game.round_view().guesses.len(), 1);
    assert_eq!(game.view().guesses, view.guesses);
}
