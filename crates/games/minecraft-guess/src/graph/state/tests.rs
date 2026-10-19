use super::*;
use std::collections::HashSet;

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
            assert_eq!(view.nodes.len(), usize::from(nodes));
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
                let neighbours: HashSet<_> = view
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
                assert!((2..=4).contains(&neighbours.len()));
                if node.kind.is_none() {
                    assert!(include_tags);
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
fn turns_node_histories_cooldown_five_points_and_tag_completion() {
    let mut game = GraphGame::new(vec![0, 1], GraphRules::default(), 5).unwrap();
    assert_eq!(game.select(1, 2), Err(GameError::NotPlayersTurn));
    game.select(0, 2).unwrap();
    assert_eq!(game.select(0, 3), Err(GameError::NodeSwitchCoolingDown));
    game.advance_time(Duration::from_millis(600));
    game.select(0, 1).unwrap();
    let answer = game.entry(0).unwrap().clone();
    let wrong = crate::Catalogue::bundled()
        .search(answer.kind, "")
        .find(|entry| entry.id != answer.id)
        .unwrap();
    game.guess(0, &wrong.id).unwrap();
    assert_eq!(game.current_player(), Some(1));
    assert!(game.take_score_gains().is_empty());
    assert_eq!(game.round_view().guesses.len(), 1);
    game.guess(1, &answer.id).unwrap();
    assert_eq!(game.take_score_gains(), [(1, 5)]);
    assert!(!game.finished());
    for index in 1..game.nodes.len() {
        let Some(answer) = game.entry(index).cloned() else {
            continue;
        };
        let player = game.current_player().unwrap();
        game.advance_time(Duration::from_millis(600));
        game.select(player, index as u16 + 1).unwrap();
        game.guess(player, &answer.id).unwrap();
        assert_eq!(game.take_score_gains(), [(player, 5)]);
    }
    assert!(game.finished());
    assert!(game.view().nodes.iter().all(GraphNodeView::revealed));
    assert!(
        game.view()
            .edges
            .iter()
            .all(|edge| edge.active && edge.directed.is_some())
    );
    assert_eq!(game.histories[0].len(), 2);
}
