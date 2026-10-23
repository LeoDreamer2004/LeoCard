use crate::{
    Catalogue, EntryKind, GameState, GuessOptions, MatchState, MinecraftGuessRuleSet,
    TurnBasedRules,
};

#[test]
fn match_statistics_include_each_small_game_and_only_accepted_actions() {
    let rules = MinecraftGuessRuleSet {
        turn_based: TurnBasedRules {
            options: GuessOptions {
                candidate_hints: false,
                allow_unknown_guesses: true,
                ..MinecraftGuessRuleSet::default().turn_based.options
            },
            ..MinecraftGuessRuleSet::default().turn_based
        },
        ..Default::default()
    };
    let answer = Catalogue::bundled()
        .find(EntryKind::Block, "minecraft:stone")
        .unwrap()
        .clone();
    let mut game = GameState::new(answer, vec![0, 1], rules);
    let mut state = MatchState::new(vec![0, 1], rules);
    assert!(game.guess(1, "不存在").is_err());
    game.skip(0).unwrap();
    game.guess(1, "minecraft:granite").unwrap();
    game.guess(0, "不存在").unwrap();
    game.guess(1, "minecraft:diorite").unwrap();
    game.guess(0, "minecraft:stone").unwrap();
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
    let answer = Catalogue::bundled()
        .find(EntryKind::Item, "minecraft:stick")
        .unwrap()
        .clone();
    let mut game = GameState::new(answer, vec![1, 0], MinecraftGuessRuleSet::default());
    game.guess(1, "minecraft:stick").unwrap();
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
