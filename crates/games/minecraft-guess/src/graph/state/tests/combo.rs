use super::super::GraphGame;
use super::fixtures::{graph, points, select};
use crate::graph::data::{GraphData, GraphEdge};
use crate::{GameError, GraphRelation};

fn game() -> GraphGame {
    graph(
        &[
            "minecraft:blue_wool",
            "minecraft:purple_wool",
            "minecraft:orange_wool",
            "minecraft:lime_wool",
            "minecraft:gray_wool",
        ],
        &[(0, 1), (2, 0), (1, 3)],
    )
}

#[test]
fn correct_answers_keep_the_turn_and_follow_only_the_last_correct_nodes_neighbours() {
    let mut game = game();
    game.guess(0, "minecraft:blue_wool").unwrap();
    assert_eq!(game.current_player(), Some(0));
    assert_eq!(game.view().combo.as_ref().unwrap().count, 1);
    assert_eq!(game.view().combo.as_ref().unwrap().targets, [2, 3]);
    assert!(game.view().can_guess_node(2));
    assert!(game.view().can_guess_node(3));
    for number in [0, 1, 4, 5, 6] {
        assert!(!game.view().can_guess_node(number));
    }
    let before = game.view();
    assert_eq!(game.select(0, 4), Err(GameError::ComboRequiresNeighbour));
    assert_eq!(game.select(1, 2), Err(GameError::NotPlayersTurn));
    assert_eq!(game.view(), before);

    // A correct answer clears the old selection cooldown for the next combo choice.
    game.select(0, 2).unwrap();
    game.guess(0, "minecraft:purple_wool").unwrap();
    assert_eq!(game.view().combo.as_ref().unwrap().targets, [4]);
    assert_eq!(game.select(0, 3), Err(GameError::ComboRequiresNeighbour));
    game.select(0, 4).unwrap();
    game.guess(0, "minecraft:lime_wool").unwrap();

    // The last node's sole neighbour was revealed, so disconnected targets are allowed.
    assert_eq!(game.view().combo.as_ref().unwrap().targets, [3, 5]);
    game.select(0, 5).unwrap();
    game.guess(0, "minecraft:gray_wool").unwrap();
    assert_eq!(game.view().combo.as_ref().unwrap().targets, [3]);
    game.select(0, 3).unwrap();
    game.guess(0, "minecraft:orange_wool").unwrap();
    assert!(game.finished());
    assert_eq!(game.current_player(), None);
    assert!(game.view().combo.is_none());
    assert_eq!(
        game.view()
            .guesses
            .iter()
            .map(|guess| guess.combo)
            .collect::<Vec<_>>(),
        [1, 2, 3, 4, 5]
    );
    let gains = game.take_score_gains();
    assert_eq!(gains.len(), 5);
    assert!(
        gains
            .iter()
            .all(|&(player, points)| player == 0 && points > 1000)
    );
}

#[test]
fn rejected_actions_preserve_combos_but_incorrect_green_guesses_and_skips_end_them() {
    let mut game = game();
    game.guess(0, "minecraft:blue_wool").unwrap();
    assert!(points(&mut game, 0) > 2000);
    select(&mut game, 2);
    let before = game.view();
    assert_eq!(game.guess(0, "不存在的条目"), Err(GameError::UnknownEntry));
    assert_eq!(game.view(), before);
    game.guess(0, "minecraft:red_wool").unwrap();
    assert!(game.round_view().guesses[0].result.all_matching());
    assert!(points(&mut game, 0) > 1000);
    assert!(game.view().combo.is_none());
    assert_eq!(game.view().guesses.last().unwrap().combo, 0);
    assert_eq!(game.current_player(), Some(1));

    select(&mut game, 5);
    game.guess(1, "minecraft:gray_wool").unwrap();
    assert_eq!(game.view().guesses.last().unwrap().combo, 1);
    game.skip(1).unwrap();
    assert_eq!(game.current_player(), Some(0));
    assert!(game.view().combo.is_none());
    select(&mut game, 4);
    game.guess(0, "minecraft:lime_wool").unwrap();
    assert_eq!(game.view().guesses.last().unwrap().combo, 1);
    game.reveal_all().unwrap();
    assert!(game.view().combo.is_none());
}

#[test]
fn guessing_also_checks_combo_eligibility_and_cannot_bypass_selection_validation() {
    let mut game = game();
    game.guess(0, "minecraft:blue_wool").unwrap();
    game.take_score_gains();
    game.selected = 3;
    let before = game.view();
    assert_eq!(
        game.guess(0, "minecraft:lime_wool"),
        Err(GameError::ComboRequiresNeighbour)
    );
    assert_eq!(game.view(), before);
    assert!(game.take_score_gains().is_empty());
    assert_eq!(game.current_player(), Some(0));
}

#[test]
fn tags_are_not_combo_targets_or_paths_and_auto_reveals_do_not_extend_the_streak() {
    let mut game = graph(
        &[
            "minecraft:blue_wool",
            "minecraft:purple_wool",
            "minecraft:orange_wool",
        ],
        &[(0, 1)],
    );
    let tag = GraphData::bundled()
        .nodes
        .iter()
        .position(|node| node.entry.is_none())
        .unwrap();
    game.nodes.push(tag);
    game.revealed.push(false);
    for index in [0, 2] {
        game.edges.push(GraphEdge {
            from: game.nodes[index],
            to: tag,
            kind: GraphRelation::MemberOf,
            directed: true,
        });
    }
    game.guess(0, "minecraft:blue_wool").unwrap();
    assert_eq!(game.view().combo.as_ref().unwrap().targets, [2]);
    assert_eq!(game.select(0, 3), Err(GameError::ComboRequiresNeighbour));
    assert_eq!(game.select(0, 4), Err(GameError::ComboRequiresNeighbour));
    game.select(0, 2).unwrap();
    game.guess(0, "minecraft:purple_wool").unwrap();
    assert_eq!(game.view().combo.as_ref().unwrap().targets, [3]);
    game.select(0, 3).unwrap();
    game.guess(0, "minecraft:orange_wool").unwrap();
    assert!(game.finished());
    assert!(game.view().nodes[3].revealed());
    assert_eq!(game.view().guesses.len(), 3);
    assert_eq!(game.view().guesses.last().unwrap().combo, 3);
    let gains = game.take_score_gains();
    assert_eq!(gains.len(), 3);
    assert!(
        gains
            .iter()
            .all(|&(player, points)| player == 0 && points > 1000)
    );
}
