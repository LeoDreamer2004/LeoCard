use super::fixtures::{graph, select};
use crate::{GameError, GuessCategoryCount};

#[test]
fn graph_archive_counts_real_answers_actions_and_each_node_kind() {
    let mut game = graph(
        &["minecraft:blue_wool", "minecraft:stick", "minecraft:zombie"],
        &[(0, 1), (1, 2)],
    );
    game.rules.options.allow_unknown_guesses = true;
    game.rules.options.candidate_hints = false;
    assert_eq!(
        game.guess(1, "minecraft:blue_wool"),
        Err(GameError::NotPlayersTurn)
    );
    game.guess(0, "minecraft:red_wool").unwrap();
    assert!(game.guesses[0].guess.result.all_matching());
    assert_eq!(
        game.guess(1, "minecraft:red_wool"),
        Err(GameError::AlreadyGuessed)
    );
    game.skip(1).unwrap();
    game.guess(0, "不存在").unwrap();
    game.guess(1, "minecraft:blue_wool").unwrap();
    select(&mut game, 2);
    game.guess(1, "minecraft:stick").unwrap();
    select(&mut game, 3);
    game.guess(1, "minecraft:zombie").unwrap();
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
        GuessCategoryCount { games: 3, wins: 0 }
    );
    assert_eq!(first.non_worse_guesses, 1);
    let second = stats[1].1;
    assert_eq!(
        (second.known_guesses, second.non_worse_guesses, second.skips),
        (3, 3, 1)
    );
    assert_eq!(
        second.categories.block,
        GuessCategoryCount { games: 1, wins: 1 }
    );
    assert_eq!(
        second.categories.item,
        GuessCategoryCount { games: 1, wins: 1 }
    );
    assert_eq!(
        second.categories.mob,
        GuessCategoryCount { games: 1, wins: 1 }
    );
}

#[test]
fn developer_reveal_does_not_invent_solved_nodes() {
    let mut game = graph(&["minecraft:stone", "minecraft:stick"], &[(0, 1)]);
    game.reveal_all().unwrap();
    for (_, stats) in game.statistics() {
        assert_eq!(
            stats.categories.total(),
            GuessCategoryCount { games: 2, wins: 0 }
        );
        assert_eq!(stats.known_guesses, 0);
    }
}
