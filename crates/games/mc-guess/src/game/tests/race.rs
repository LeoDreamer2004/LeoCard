use crate::{
    McGuessCatalogue, McGuessEntryKind, McGuessGameError, McGuessGameState, McGuessMatchState,
    McGuessMode, McGuessRuleSet,
};
use leocard_game_common::PlayerId;

fn race_game() -> McGuessGameState {
    let answer = McGuessCatalogue::bundled()
        .find(McGuessEntryKind::Block, "minecraft:blue_wool")
        .unwrap()
        .clone();
    McGuessGameState::new(
        answer,
        vec![PlayerId(0), PlayerId(1)],
        McGuessRuleSet {
            mode: McGuessMode::Race,
            ..Default::default()
        },
    )
}

#[test]
fn all_players_can_repeat_actions_but_entries_and_green_bonus_are_unique() {
    let mut game = race_game();
    assert_eq!(game.current_player(), None);
    assert!(game.view().can_guess(PlayerId(1), McGuessMode::Race));
    game.guess(PlayerId(1), "minecraft:red_wool").unwrap();
    assert_eq!(game.take_score_gains(), vec![(PlayerId(1), 3)]);
    let before = game.view();
    assert_eq!(
        game.guess(PlayerId(0), "红色羊毛"),
        Err(McGuessGameError::AlreadyGuessed)
    );
    assert_eq!(
        game.guess(PlayerId(9), "minecraft:stone"),
        Err(McGuessGameError::NotPlayersTurn)
    );
    assert_eq!(game.skip(PlayerId(1)), Err(McGuessGameError::CannotSkip));
    assert_eq!(game.view(), before);
    game.guess(PlayerId(1), "minecraft:green_wool").unwrap();
    assert!(game.take_score_gains().is_empty());
    game.guess(PlayerId(0), "minecraft:blue_wool").unwrap();
    assert_eq!(game.take_score_gains(), vec![(PlayerId(0), 5)]);
    assert_eq!(game.view().winner, Some(PlayerId(0)));
    assert!(!game.view().can_guess(PlayerId(1), McGuessMode::Race));
    assert_eq!(
        game.guess(PlayerId(1), "minecraft:stone"),
        Err(McGuessGameError::Finished)
    );
}

#[test]
fn direct_answer_is_five_points_and_expiry_does_not_invent_skips_or_scores() {
    let mut game = race_game();
    game.guess(PlayerId(1), "minecraft:blue_wool").unwrap();
    assert_eq!(game.take_score_gains(), vec![(PlayerId(1), 5)]);
    let mut game = race_game();
    game.guess(PlayerId(1), "minecraft:stone").unwrap();
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
            let mut rules = McGuessRuleSet {
                mode: McGuessMode::Race,
                ..Default::default()
            };
            rules.race.games = games;
            let mut state = McGuessMatchState::new((0..players).map(PlayerId).collect(), rules);
            for index in 0..games {
                assert_eq!(state.progress().game_index, index);
                assert_eq!(state.progress().total_games(), games);
                assert_eq!(state.progress().is_last_game(), index + 1 == games);
                assert_eq!(state.advance(), index + 1 < games);
            }
        }
    }
}
