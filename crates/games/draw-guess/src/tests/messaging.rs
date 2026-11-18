use crate::{DrawGuessError, DrawGuessGame, DrawGuessPhase, DrawGuessRuleSet};
use leocard_game_common::PlayerId;

fn drawing_game() -> DrawGuessGame {
    let mut game = DrawGuessGame::new(
        DrawGuessRuleSet {
            rounds: 1,
            drawing_seconds: 30,
        },
        (0..4).map(PlayerId).collect(),
        ["苹果", "香蕉", "橘子"].map(str::to_owned).to_vec(),
    )
    .unwrap();
    game.choose_word(PlayerId(0), 0).unwrap();
    game
}

#[test]
fn solved_chat_is_private_and_never_awards_points_again() {
    let mut game = drawing_game();
    game.guess(PlayerId(1), "苹果").unwrap();
    game.guess(PlayerId(2), "香蕉").unwrap();
    let scores = game.players().to_vec();
    game.advance_ms(350);
    let chat = game.guess(PlayerId(1), "答案是苹果").unwrap();
    assert!(chat.private);
    assert!(!chat.correct);
    assert_eq!(chat.points, 0);
    assert_eq!(game.players(), scores);
    for recipient in [PlayerId(0), PlayerId(1)] {
        assert!(game.view(recipient).history.contains(&chat));
    }
    for recipient in [PlayerId(2), PlayerId(3), PlayerId(255)] {
        let view = game.view(recipient);
        assert!(view.history.iter().all(|message| !message.private));
        assert!(view.history.iter().any(|message| message.text == "香蕉"));
    }
    game.advance_ms(350);
    let repeat_answer = game.guess(PlayerId(1), "苹果").unwrap();
    assert!(repeat_answer.private);
    assert!(!repeat_answer.correct);
    assert_eq!(game.players(), scores);
    game.guess(PlayerId(2), "苹果").unwrap();
    assert!(game.view(PlayerId(2)).history.contains(&chat));
    game.advance_ms(350);
    assert!(game.guess(PlayerId(1), "!!!").unwrap().private);
    game.advance_ms(30_000);
    assert_eq!(game.phase(), DrawGuessPhase::Reveal);
    for recipient in [PlayerId(3), PlayerId(255)] {
        assert!(
            game.view(recipient)
                .history
                .iter()
                .all(|message| !message.private)
        );
    }
}

#[test]
fn public_and_private_messages_are_limited_to_twenty_characters() {
    let mut game = drawing_game();
    let accepted = "猫".repeat(20);
    let rejected = "猫".repeat(21);
    assert!(game.guess(PlayerId(1), &accepted).is_ok());
    assert_eq!(
        game.guess(PlayerId(1), &rejected),
        Err(DrawGuessError::InvalidGuess)
    );
    game.advance_ms(350);
    game.guess(PlayerId(1), "苹果").unwrap();
    game.advance_ms(350);
    let chat = game.guess(PlayerId(1), &accepted).unwrap();
    assert!(chat.private);
    assert_eq!(chat.text.chars().count(), 20);
    let before = game.view(PlayerId(1));
    assert_eq!(
        game.guess(PlayerId(1), &rejected),
        Err(DrawGuessError::InvalidGuess)
    );
    assert_eq!(game.view(PlayerId(1)), before);
}
