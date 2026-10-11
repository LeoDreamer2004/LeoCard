use crate::{
    Catalogue, EntryKind, GameError, GameState, GuessPool, GuessResult, MAX_GUESS_CHARACTERS,
    MinecraftGuessRuleSet,
};

#[test]
fn unknown_guesses_consume_turns_without_attributes_or_answer_leaks() {
    let answer = Catalogue::bundled()
        .find(EntryKind::Block, "minecraft:oak_log")
        .unwrap()
        .clone();
    let rules = MinecraftGuessRuleSet {
        candidate_hints: false,
        allow_unknown_guesses: true,
        ..MinecraftGuessRuleSet::default()
    };
    let mut game = GameState::new(answer.clone(), vec![0, 1], rules);
    game.skip(0).unwrap();
    game.guess(1, "  不存在的方块  ").unwrap();
    let view = game.view();
    assert!(view.answer.is_none());
    assert!(view.skipped_players.is_empty());
    assert_eq!(view.current_player, Some(0));
    assert_eq!(view.guesses[0].player, 1);
    assert_eq!(
        view.guesses[0].result,
        GuessResult::Unknown {
            name: "不存在的方块".to_owned()
        }
    );
    assert!(view.guesses[0].result.accepts_candidate(&answer));
    for (text, error) in [
        ("不存在的方块", GameError::AlreadyGuessed),
        ("一二三四五六七八九十个", GameError::GuessTooLong),
        ("   ", GameError::InvalidGuess),
        ("名字\n换行", GameError::InvalidGuess),
    ] {
        assert_eq!(game.guess(0, text), Err(error));
        assert_eq!(game.view(), view);
    }
    game.guess(0, "一二三四五六七八九十").unwrap();
    assert_eq!(game.current_player(), Some(1));
    // Internal IDs remain usable, although their length exceeds the name input limit.
    game.guess(1, &answer.id).unwrap();
    assert_eq!(game.view().winner, Some(1));
}

#[test]
fn rules_require_no_hints_and_the_catalogue_fits_the_name_limit() {
    let rules = MinecraftGuessRuleSet::default();
    assert!(rules.is_valid());
    assert!(
        !MinecraftGuessRuleSet {
            allow_unknown_guesses: true,
            ..rules
        }
        .is_valid()
    );
    assert!(
        MinecraftGuessRuleSet {
            candidate_hints: false,
            allow_unknown_guesses: true,
            ..rules
        }
        .is_valid()
    );
    for entry in Catalogue::bundled().entries(GuessPool::Mixed) {
        assert!(
            entry.name.chars().count() <= MAX_GUESS_CHARACTERS,
            "{}",
            entry.name
        );
    }
    assert!(
        Catalogue::bundled()
            .find(EntryKind::Item, "minecraft:mojang_banner_pattern")
            .is_none()
    );
}
