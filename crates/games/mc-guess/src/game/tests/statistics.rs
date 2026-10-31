use crate::{
    McGuessCatalogue, McGuessEntryKind, McGuessGameState, McGuessMatchState, McGuessOptions,
    McGuessRuleSet, McGuessTurnBasedRules,
};
use leocard_game_common::PlayerId;

#[test]
fn match_statistics_include_each_small_game_and_only_accepted_actions() {
    let rules = McGuessRuleSet {
        turn_based: McGuessTurnBasedRules {
            options: McGuessOptions {
                candidate_hints: false,
                allow_unknown_guesses: true,
                ..McGuessRuleSet::default().turn_based.options
            },
            ..McGuessRuleSet::default().turn_based
        },
        ..Default::default()
    };
    let answer = McGuessCatalogue::bundled()
        .find(McGuessEntryKind::Block, "minecraft:stone")
        .unwrap()
        .clone();
    let mut game = McGuessGameState::new(answer, vec![PlayerId(0), PlayerId(1)], rules);
    let mut state = McGuessMatchState::new(vec![PlayerId(0), PlayerId(1)], rules);
    assert!(game.guess(PlayerId(1), "不存在").is_err());
    game.skip(PlayerId(0)).unwrap();
    game.guess(PlayerId(1), "minecraft:granite").unwrap();
    game.guess(PlayerId(0), "不存在").unwrap();
    game.guess(PlayerId(1), "minecraft:diorite").unwrap();
    game.guess(PlayerId(0), "minecraft:stone").unwrap();
    state.finish_round(&mut game);
    let first = *state.statistics().next().unwrap().1;
    assert_eq!(
        (
            first.categories.block.games,
            first.categories.block.wins,
            first.skips
        ),
        (1, 1, 1)
    );
    assert_eq!(
        (
            first.known_guesses,
            first.unknown_guesses,
            first.unknown_enabled_guesses
        ),
        (1, 1, 2)
    );
    assert_eq!(first.non_worse_guesses, 1);
    let other = *state.statistics().nth(1).unwrap().1;
    // Granite and diorite give equally close feedback, so both count as non-worse guesses.
    assert_eq!((other.known_guesses, other.non_worse_guesses), (2, 2));
    assert_eq!(other.categories.block.games, 1);
    assert!(state.advance());
    let answer = McGuessCatalogue::bundled()
        .find(McGuessEntryKind::Item, "minecraft:stick")
        .unwrap()
        .clone();
    let mut game = McGuessGameState::new(
        answer,
        vec![PlayerId(1), PlayerId(0)],
        McGuessRuleSet::default(),
    );
    game.guess(PlayerId(1), "minecraft:stick").unwrap();
    state.finish_round(&mut game);
    let other = *state.statistics().nth(1).unwrap().1;
    assert_eq!(
        (
            other.categories.block.games,
            other.categories.item.games,
            other.categories.item.wins
        ),
        (1, 1, 1)
    );
    assert_eq!((other.known_guesses, other.unknown_enabled_guesses), (3, 2));
}
