use super::fixtures::{graph, select};
use crate::{McGuessCategoryCount, McGuessGameError};
use leocard_game_common::PlayerId;

#[test]
fn graph_archive_counts_real_answers_actions_and_each_node_kind() {
    let mut game = graph(
        &["minecraft:blue_wool", "minecraft:stick", "minecraft:zombie"],
        &[(0, 1), (1, 2)],
    );
    game.rules.options.allow_unknown_guesses = true;
    game.rules.options.candidate_hints = false;
    assert_eq!(
        game.guess(PlayerId(1), "minecraft:blue_wool"),
        Err(McGuessGameError::NotPlayersTurn)
    );
    game.guess(PlayerId(0), "minecraft:red_wool").unwrap();
    assert!(game.guesses[0].guess.result.all_matching());
    assert_eq!(
        game.guess(PlayerId(1), "minecraft:red_wool"),
        Err(McGuessGameError::AlreadyGuessed)
    );
    game.skip(PlayerId(1)).unwrap();
    game.guess(PlayerId(0), "不存在").unwrap();
    game.guess(PlayerId(1), "minecraft:blue_wool").unwrap();
    select(&mut game, 2);
    game.guess(PlayerId(1), "minecraft:stick").unwrap();
    select(&mut game, 3);
    game.guess(PlayerId(1), "minecraft:zombie").unwrap();
    assert!(game.finished());
    let stats: Vec<_> = game.statistics().collect();
    let first = stats[0].1;
    assert_eq!(
        (
            first.known_guesses,
            first.unknown_guesses,
            first.unknown_enabled_guesses
        ),
        (1, 1, 2)
    );
    assert_eq!(
        first.categories.total(),
        McGuessCategoryCount { games: 3, wins: 0 }
    );
    assert_eq!(first.non_worse_guesses, 1);
    let second = stats[1].1;
    assert_eq!(
        (second.known_guesses, second.non_worse_guesses, second.skips),
        (3, 3, 1)
    );
    assert_eq!(
        second.categories.block,
        McGuessCategoryCount { games: 1, wins: 1 }
    );
    assert_eq!(
        second.categories.item,
        McGuessCategoryCount { games: 1, wins: 1 }
    );
    assert_eq!(
        second.categories.mob,
        McGuessCategoryCount { games: 1, wins: 1 }
    );
}

#[test]
fn developer_reveal_does_not_invent_solved_nodes() {
    let mut game = graph(&["minecraft:stone", "minecraft:stick"], &[(0, 1)]);
    game.reveal_all().unwrap();
    for (_, stats) in game.statistics() {
        assert_eq!(
            stats.categories.total(),
            McGuessCategoryCount { games: 2, wins: 0 }
        );
        assert_eq!(stats.known_guesses, 0);
    }
}

#[test]
fn consecutive_skips_finish_across_nodes_without_inventing_answers() {
    let mut game = graph(&["minecraft:stone", "minecraft:stick"], &[(0, 1)]);
    game.skip(PlayerId(0)).unwrap();
    assert!(!game.finished());
    game.guess(PlayerId(1), "minecraft:dirt").unwrap();
    game.take_score_gains();
    assert_eq!(
        game.skip(PlayerId(1)),
        Err(McGuessGameError::NotPlayersTurn)
    );
    game.skip(PlayerId(0)).unwrap();
    assert!(!game.finished());
    select(&mut game, 2);
    game.skip(PlayerId(1)).unwrap();
    assert!(game.finished());
    assert_eq!(game.current_player(), None);
    let view = game.view();
    assert!(view.nodes.iter().all(|node| node.revealed()));
    assert!(view.edges.iter().all(|edge| edge.active));
    assert_eq!(view.guesses.len(), 1);
    assert!(game.take_score_gains().is_empty());
    assert!(game.take_hand_statistics().is_empty());
    for (_, stats) in game.statistics() {
        assert_eq!(
            stats.categories.total(),
            McGuessCategoryCount { games: 2, wins: 0 }
        );
    }
    assert_eq!(game.skip(PlayerId(0)), Err(McGuessGameError::Finished));
}
