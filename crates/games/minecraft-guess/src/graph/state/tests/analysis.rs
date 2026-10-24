use super::fixtures::{graph, select};
use crate::GameError;

#[test]
fn graph_actions_report_real_guesses_and_node_completion_once() {
    let mut game = graph(&["minecraft:blue_wool", "minecraft:stick"], &[(0, 1)]);
    game.rules.options.allow_unknown_guesses = true;
    game.rules.options.candidate_hints = false;
    assert_eq!(
        game.guess(1, "minecraft:blue_wool"),
        Err(GameError::NotPlayersTurn)
    );
    assert!(game.take_action_statistics().is_empty());
    game.guess(0, "minecraft:red_wool").unwrap();
    let action = game.take_action_statistics().pop().unwrap();
    assert!(action.wrong_all_matching && action.correct_entry.is_none());
    assert!(game.take_hand_statistics().is_empty());
    game.skip(1).unwrap();
    assert_eq!(game.take_action_statistics().pop().unwrap().skip_run, 1);
    game.guess(0, "不存在").unwrap();
    let action = game.take_action_statistics().pop().unwrap();
    assert!(action.unknown && action.unknown_run == 1);
    game.guess(1, "minecraft:blue_wool").unwrap();
    let action = game.take_action_statistics().pop().unwrap();
    assert!(action.won_after_skip);
    assert_eq!(action.correct_entry.unwrap().id, "minecraft:blue_wool");
    let hands = game.take_hand_statistics();
    assert_eq!(hands.len(), 2);
    assert!(!hands[0].consistent && hands[1].consistent);
    assert!(game.take_action_statistics().is_empty() && game.take_hand_statistics().is_empty());
    // Activating the remaining graph never fabricates answer or consistency progress.
    game.reveal_all().unwrap();
    assert!(game.take_action_statistics().is_empty() && game.take_hand_statistics().is_empty());
}

#[test]
fn matching_runs_and_first_cycle_guesses_belong_to_the_selected_node() {
    let mut game = graph(&["minecraft:blue_wool", "minecraft:stick"], &[(0, 1)]);
    for name in [
        "red",
        "green",
        "black",
        "yellow",
        "orange",
        "pink",
        "gray",
        "light_blue",
        "cyan",
        "purple",
        "brown",
    ] {
        let player = game.current_player().unwrap();
        game.guess(player, &format!("minecraft:{name}_wool"))
            .unwrap();
        assert!(
            game.take_action_statistics()
                .pop()
                .unwrap()
                .wrong_all_matching
        );
    }
    select(&mut game, 2);
    let player = game.current_player().unwrap();
    game.guess(player, "minecraft:stick").unwrap();
    let action = game.take_action_statistics().pop().unwrap();
    assert!(!action.opening_win && action.first_cycle_win && !action.ended_matching_run);
    select(&mut game, 1);
    game.guess(player, "minecraft:blue_wool").unwrap();
    let action = game.take_action_statistics().pop().unwrap();
    assert!(
        action.ended_matching_run
            && action.long_search_win
            && !action.opening_win
            && !action.first_cycle_win
    );
}

#[test]
fn every_solved_node_emits_one_consistency_result_for_each_player() {
    let ids = [
        "minecraft:stone",
        "minecraft:dirt",
        "minecraft:cobblestone",
        "minecraft:stick",
        "minecraft:zombie",
    ];
    let mut game = graph(&ids, &[(0, 1), (1, 2), (2, 3), (3, 4)]);
    for (index, id) in ids.into_iter().enumerate() {
        select(&mut game, index as u16 + 1);
        game.guess(0, id).unwrap();
        let hands = game.take_hand_statistics();
        assert_eq!(hands.len(), 2);
        assert!(hands[0].consistent && !hands[1].consistent);
        assert!(!hands[0].round_sweep && !hands[0].match_sweep);
        assert!(game.take_hand_statistics().is_empty());
    }
    assert!(game.finished());
}
