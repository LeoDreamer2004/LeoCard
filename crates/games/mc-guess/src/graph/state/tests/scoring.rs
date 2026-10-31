use super::super::McGuessGraphGame;
use super::fixtures::{graph, points, select};
use crate::McGuessGameError;
use leocard_game_common::PlayerId;

fn game() -> McGuessGraphGame {
    graph(&["minecraft:blue_wool", "minecraft:purple_wool"], &[])
}

#[test]
fn first_green_bonus_is_per_node_and_cannot_be_repeated_by_switching_or_aliases() {
    let mut game = game();
    game.guess(PlayerId(0), "minecraft:red_wool").unwrap();
    let first = points(&mut game, PlayerId(0));
    assert!(first > 1000);
    assert!(game.round_view().guesses[0].result.all_matching());
    assert!(!game.view().nodes[0].revealed());
    assert!(game.view().combo.is_none());
    assert_eq!(game.view().guesses[0].combo, 0);
    assert_eq!(
        game.guess(PlayerId(1), "红色羊毛"),
        Err(McGuessGameError::AlreadyGuessed)
    );
    assert!(game.take_score_gains().is_empty());
    assert_eq!(game.current_player(), Some(PlayerId(1)));
    game.guess(PlayerId(1), "minecraft:green_wool").unwrap();
    let follow = points(&mut game, PlayerId(1));
    assert!(follow.abs_diff((f64::from(first) * 0.3 * 0.95).round() as u32) <= 1);

    select(&mut game, 2);
    game.guess(PlayerId(0), "minecraft:red_wool").unwrap();
    let other_first = points(&mut game, PlayerId(0));
    assert!(other_first > 1000);
    game.guess(PlayerId(1), "minecraft:green_wool").unwrap();
    assert!(
        points(&mut game, PlayerId(1))
            .abs_diff((f64::from(other_first) * 0.3 * 0.95).round() as u32)
            <= 1
    );

    select(&mut game, 1);
    game.guess(PlayerId(0), "minecraft:yellow_wool").unwrap();
    let expected = (f64::from(first) * 0.3 * 0.95_f64.powi(2)).round() as u32;
    assert!(points(&mut game, PlayerId(0)).abs_diff(expected) <= 1);
    game.guess(PlayerId(1), "minecraft:blue_wool").unwrap();
    let expected = (f64::from(first) * 2.0 * 0.95_f64.powi(3)).round() as u32;
    assert!(points(&mut game, PlayerId(1)).abs_diff(expected) <= 1);
    assert!(game.view().nodes[0].revealed());
    assert!(!game.finished());
    assert_eq!(
        game.guess(PlayerId(1), "minecraft:orange_wool"),
        Err(McGuessGameError::NodeAlreadyRevealed)
    );
    assert!(game.take_score_gains().is_empty());

    select(&mut game, 2);
    game.guess(PlayerId(1), "minecraft:orange_wool").unwrap();
    let follow_after_answer = points(&mut game, PlayerId(1));
    // Only global blindness changes: half of the two nodes is now revealed.
    let expected = (f64::from(other_first) / 1.3 * 1.15 * 0.3 * 0.95_f64.powi(2)).round() as u32;
    assert!(follow_after_answer.abs_diff(expected) <= 1);
    assert!(game.view().combo.is_none());
    assert_eq!(game.current_player(), Some(PlayerId(0)));
    game.guess(PlayerId(0), "minecraft:purple_wool").unwrap();
    assert!(points(&mut game, PlayerId(0)) > 2000);
    assert_eq!(game.view().guesses.last().unwrap().combo, 1);
    assert!(game.finished());
    assert!(game.take_score_gains().is_empty());
}

#[test]
fn direct_answers_earn_double_value_and_discount_the_second_combo() {
    let mut game = game();
    game.guess(PlayerId(0), "minecraft:blue_wool").unwrap();
    let first = points(&mut game, PlayerId(0));
    assert!(first > 2000);
    select(&mut game, 2);
    game.guess(PlayerId(0), "minecraft:purple_wool").unwrap();
    let second = points(&mut game, PlayerId(0));
    assert!(second > 2000 && second < first);
    assert!(game.finished());
}

#[test]
fn wrong_unknown_and_skipped_guesses_do_not_consume_the_first_green_bonus() {
    let mut game = game();
    game.rules.options.allow_unknown_guesses = true;
    game.rules.options.candidate_hints = false;
    game.guess(PlayerId(0), "minecraft:blue_wool").unwrap();
    game.take_score_gains();
    select(&mut game, 2);
    game.guess(PlayerId(0), "minecraft:stone").unwrap();
    assert!(!game.round_view().guesses[0].result.all_matching());
    assert!((100..250).contains(&points(&mut game, PlayerId(0))));
    assert_eq!(game.current_player(), Some(PlayerId(1)));
    assert!(game.view().combo.is_none());
    assert_eq!(game.view().guesses.last().unwrap().combo, 0);
    game.guess(PlayerId(1), "未知条目").unwrap();
    assert!(game.take_score_gains().is_empty());
    game.skip(PlayerId(0)).unwrap();
    assert!(game.take_score_gains().is_empty());
    game.guess(PlayerId(1), "minecraft:red_wool").unwrap();
    assert!(points(&mut game, PlayerId(1)) > 1000);
    assert!(!game.finished());
}
