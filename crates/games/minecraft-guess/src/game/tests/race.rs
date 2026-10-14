use crate::{
    Catalogue, EntryKind, GameError, GameState, GuessMode, MatchState, MinecraftGuessRuleSet,
};

fn race_game() -> GameState {
    let answer = Catalogue::bundled()
        .find(EntryKind::Block, "minecraft:blue_wool")
        .unwrap()
        .clone();
    GameState::new(
        answer,
        vec![0, 1],
        MinecraftGuessRuleSet {
            mode: GuessMode::Race,
            ..Default::default()
        },
    )
}

#[test]
fn all_players_can_repeat_actions_but_entries_and_green_bonus_are_unique() {
    let mut game = race_game();
    assert_eq!(game.current_player(), None);
    assert!(game.view().can_guess(1, GuessMode::Race));
    game.guess(1, "minecraft:red_wool").unwrap();
    assert_eq!(game.take_score_gains(), vec![(1, 3)]);
    let before = game.view();
    assert_eq!(game.guess(0, "红色羊毛"), Err(GameError::AlreadyGuessed));
    assert_eq!(
        game.guess(9, "minecraft:stone"),
        Err(GameError::NotPlayersTurn)
    );
    assert_eq!(game.skip(1), Err(GameError::CannotSkip));
    assert_eq!(game.view(), before);
    game.guess(1, "minecraft:green_wool").unwrap();
    assert!(game.take_score_gains().is_empty());
    game.guess(0, "minecraft:blue_wool").unwrap();
    assert_eq!(game.take_score_gains(), vec![(0, 5)]);
    assert_eq!(game.view().winner, Some(0));
    assert!(!game.view().can_guess(1, GuessMode::Race));
    assert_eq!(game.guess(1, "minecraft:stone"), Err(GameError::Finished));
}

#[test]
fn direct_answer_is_five_points_and_expiry_does_not_invent_skips_or_scores() {
    let mut game = race_game();
    game.guess(1, "minecraft:blue_wool").unwrap();
    assert_eq!(game.take_score_gains(), vec![(1, 5)]);
    let mut game = race_game();
    game.guess(1, "minecraft:stone").unwrap();
    assert!(game.take_score_gains().is_empty());
    game.expire().unwrap();
    let view = game.view();
    assert!(view.finished());
    assert_eq!(view.answer.unwrap().id, "minecraft:blue_wool");
    assert_eq!(view.winner, None);
    assert!(view.skipped_players.is_empty());
    assert!(game.take_score_gains().is_empty());
    assert!(game.statistics().all(|(_, stats)| stats.skips == 0));
}

#[test]
fn race_length_counts_questions_regardless_of_number_of_players() {
    for players in [2, 8] {
        for games in [3, 5, 10] {
            let mut rules = MinecraftGuessRuleSet {
                mode: GuessMode::Race,
                ..Default::default()
            };
            rules.race.games = games;
            let mut state = MatchState::new((0..players).collect(), rules);
            for index in 0..games {
                assert_eq!(state.progress().game_index, index);
                assert_eq!(state.progress().total_games(), games);
                assert_eq!(state.progress().is_last_game(), index + 1 == games);
                assert_eq!(state.advance(), index + 1 < games);
            }
        }
    }
}
