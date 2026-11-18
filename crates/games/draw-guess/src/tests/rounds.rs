use crate::{
    CanvasPoint, DrawAction, DrawGuessError, DrawGuessGame, DrawGuessPhase, DrawGuessRuleSet, Ink,
    WordLibrary, word_bank,
};
use leocard_game_common::PlayerId;

fn game() -> DrawGuessGame {
    DrawGuessGame::new(
        DrawGuessRuleSet {
            rounds: 1,
            drawing_seconds: 30,
        },
        vec![PlayerId(2), PlayerId(7), PlayerId(9)],
        word_bank(),
    )
    .unwrap()
}

#[test]
fn a_match_requires_two_to_eight_distinct_players() {
    for count in [1, 9] {
        assert!(matches!(
            DrawGuessGame::new(
                DrawGuessRuleSet::default(),
                (0..count).map(PlayerId).collect(),
                word_bank(),
            ),
            Err(DrawGuessError::InvalidPlayers)
        ));
    }
    assert!(matches!(
        DrawGuessGame::new(
            DrawGuessRuleSet::default(),
            vec![PlayerId(0), PlayerId(0)],
            word_bank(),
        ),
        Err(DrawGuessError::InvalidPlayers)
    ));
}

#[test]
fn only_the_artist_receives_choices_and_unguessed_answers_remain_private() {
    let mut game = game();
    let artist = PlayerId(2);
    assert_eq!(game.view(artist).choices.len(), 3);
    assert!(game.view(PlayerId(7)).choices.is_empty());
    assert!(game.view(PlayerId(255)).choices.is_empty());
    assert_eq!(
        game.choose_word(PlayerId(7), 0),
        Err(DrawGuessError::NotArtist)
    );
    game.choose_word(artist, 1).unwrap();
    let word = game.view(artist).word.unwrap();
    assert!(game.view(PlayerId(7)).word.is_none());
    assert!(game.view(PlayerId(255)).word.is_none());
    let message = game.guess(PlayerId(7), &word).unwrap();
    assert!(message.correct);
    assert!(message.text.is_empty());
    assert_eq!(game.view(PlayerId(7)).word.as_deref(), Some(word.as_str()));
    assert!(game.view(PlayerId(9)).word.is_none());
    game.guess(PlayerId(9), &word).unwrap();
    assert_eq!(game.phase(), DrawGuessPhase::Reveal);
    assert_eq!(
        game.view(PlayerId(255)).word.as_deref(),
        Some(word.as_str())
    );
    assert_eq!(game.view(artist).players[0].score, 100);
}

#[test]
fn guesses_are_bounded_rate_limited_and_only_the_artist_can_draw() {
    let mut game = game();
    game.choose_word(PlayerId(2), 0).unwrap();
    let action = DrawAction::Begin {
        point: CanvasPoint { x: 1, y: 1 },
        color: Ink::BLACK,
        size: 4,
        eraser: false,
    };
    assert_eq!(
        game.draw(PlayerId(7), &action),
        Err(DrawGuessError::NotArtist)
    );
    game.draw(PlayerId(2), &action).unwrap();
    assert_eq!(
        game.guess(PlayerId(2), "苹果"),
        Err(DrawGuessError::CannotGuess)
    );
    assert_eq!(
        game.guess(PlayerId(255), "苹果"),
        Err(DrawGuessError::CannotGuess)
    );
    assert_eq!(
        game.guess(PlayerId(7), &"猫".repeat(21)),
        Err(DrawGuessError::InvalidGuess)
    );
    game.guess(PlayerId(7), "这是错误答案").unwrap();
    assert_eq!(
        game.guess(PlayerId(7), "另一个答案"),
        Err(DrawGuessError::GuessTooSoon)
    );
    game.advance_ms(350);
    game.guess(PlayerId(7), "另一个答案").unwrap();
}

#[test]
fn timers_rotate_artists_clear_the_canvas_and_finish_a_whole_match() {
    let mut game = game();
    game.advance_ms(15_000);
    assert_eq!(game.phase(), DrawGuessPhase::Drawing);
    game.draw(
        PlayerId(2),
        &DrawAction::Begin {
            point: CanvasPoint { x: 1, y: 1 },
            color: Ink::BLACK,
            size: 4,
            eraser: false,
        },
    )
    .unwrap();
    game.advance_ms(30_000);
    assert_eq!(game.phase(), DrawGuessPhase::Reveal);
    assert!(game.canvas().pending().is_none());
    game.advance_ms(5000);
    assert_eq!(game.artist(), Some(PlayerId(7)));
    assert_eq!(game.phase(), DrawGuessPhase::Choosing);
    assert!(game.canvas().operations().is_empty());
    game.advance_ms(100_000);
    assert_eq!(game.phase(), DrawGuessPhase::Finished);
    assert_eq!(game.artist(), None);
}

#[test]
fn disconnected_artist_is_skipped_and_reconnection_preserves_scores() {
    let mut game = game();
    game.choose_word(PlayerId(2), 0).unwrap();
    let word = game.view(PlayerId(2)).word.unwrap();
    game.guess(PlayerId(7), &word).unwrap();
    game.set_connected(PlayerId(2), false);
    assert_eq!(game.phase(), DrawGuessPhase::Reveal);
    game.advance_ms(5000);
    assert_eq!(game.artist(), Some(PlayerId(7)));
    game.set_connected(PlayerId(9), false);
    game.advance_ms(50_000);
    assert_eq!(game.phase(), DrawGuessPhase::Finished);
    game.set_connected(PlayerId(2), true);
    assert_eq!(game.view(PlayerId(2)).players[0].score, 50);
    assert_eq!(game.view(PlayerId(2)).players[1].score, 100);
}

#[test]
fn word_libraries_trim_lines_deduplicate_and_reject_invalid_or_oversized_content() {
    let library = WordLibrary::from_lines("  动物  ", " 小猫\r\n小狗\n\n小熊\n小猫\n").unwrap();
    assert_eq!(library.title, "动物");
    assert_eq!(library.words, ["小猫", "小狗", "小熊"]);
    assert!(WordLibrary::from_lines("动物", "小猫\n小猫\n小狗").is_err());
    assert!(WordLibrary::from_lines("", "小猫\n小狗\n小熊").is_err());
    assert!(WordLibrary::from_lines("动物", &format!("{}\n小狗\n小熊", "猫".repeat(21))).is_err());
    assert!(WordLibrary::from_lines("动物", "小猫\n小\t狗\n小熊").is_err());
    assert!(WordLibrary::from_lines("动物", &"猫".repeat(WordLibrary::MAX_CONTENT_BYTES)).is_err());
    assert_eq!(
        library.clone().validate().unwrap().fingerprint(),
        library.fingerprint()
    );
}
