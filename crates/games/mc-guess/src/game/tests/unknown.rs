use crate::{
    MAX_GUESS_CHARACTERS, McGuessCatalogue, McGuessEntryKind, McGuessGameError, McGuessGameState,
    McGuessOptions, McGuessPool, McGuessResult, McGuessRuleSet, McGuessTurnBasedRules,
};
use leocard_game_common::PlayerId;

#[test]
fn unknown_guesses_consume_turns_without_attributes_or_answer_leaks() {
    let answer = McGuessCatalogue::bundled()
        .find(McGuessEntryKind::Block, "minecraft:oak_log")
        .unwrap()
        .clone();
    let rules = McGuessRuleSet {
        turn_based: McGuessTurnBasedRules {
            options: McGuessOptions {
                candidate_hints: false,
                allow_unknown_guesses: true,
                ..McGuessRuleSet::default().turn_based.options
            },
            ..McGuessRuleSet::default().turn_based
        },
        ..McGuessRuleSet::default()
    };
    let mut game = McGuessGameState::new(answer.clone(), vec![PlayerId(0), PlayerId(1)], rules);
    game.skip(PlayerId(0)).unwrap();
    game.guess(PlayerId(1), "  不存在的方块  ").unwrap();
    let view = game.view();
    assert!(view.answer.is_none());
    assert!(view.skipped_players.is_empty());
    assert_eq!(view.current_player, Some(PlayerId(0)));
    assert_eq!(view.guesses[0].player, PlayerId(1));
    assert_eq!(
        view.guesses[0].result,
        McGuessResult::Unknown {
            name: "不存在的方块".to_owned()
        }
    );
    assert!(view.guesses[0].result.accepts_candidate(&answer));
    let too_long = "字".repeat(MAX_GUESS_CHARACTERS + 1);
    for (text, error) in [
        ("不存在的方块", McGuessGameError::AlreadyGuessed),
        (too_long.as_str(), McGuessGameError::GuessTooLong),
        ("   ", McGuessGameError::InvalidGuess),
        ("名字\n换行", McGuessGameError::InvalidGuess),
    ] {
        assert_eq!(game.guess(PlayerId(0), text), Err(error));
        assert_eq!(game.view(), view);
    }
    game.guess(PlayerId(0), &"字".repeat(MAX_GUESS_CHARACTERS))
        .unwrap();
    assert_eq!(game.current_player(), Some(PlayerId(1)));
    // Internal IDs remain usable, although their length exceeds the name input limit.
    game.guess(PlayerId(1), &answer.id).unwrap();
    assert_eq!(game.view().winner, Some(PlayerId(1)));
}

#[test]
fn rules_require_no_hints_and_the_catalogue_fits_the_name_limit() {
    let rules = McGuessRuleSet::default();
    assert!(rules.is_valid());
    assert!(
        !McGuessRuleSet {
            turn_based: McGuessTurnBasedRules {
                options: McGuessOptions {
                    allow_unknown_guesses: true,
                    ..rules.turn_based.options
                },
                ..rules.turn_based
            },
            ..rules
        }
        .is_valid()
    );
    assert!(
        McGuessRuleSet {
            turn_based: McGuessTurnBasedRules {
                options: McGuessOptions {
                    candidate_hints: false,
                    allow_unknown_guesses: true,
                    ..rules.turn_based.options
                },
                ..rules.turn_based
            },
            ..rules
        }
        .is_valid()
    );
    for entry in McGuessCatalogue::bundled().entries(McGuessPool::Mixed) {
        assert!(
            entry.name.chars().count() <= MAX_GUESS_CHARACTERS,
            "{}",
            entry.name
        );
    }
    assert!(
        McGuessCatalogue::bundled()
            .find(McGuessEntryKind::Item, "minecraft:mojang_banner_pattern")
            .is_some_and(|entry| entry.name == "Mojang徽标旗帜图案")
    );
}
