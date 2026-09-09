use super::super::prelude::*;

fn stack_pack_game() -> GameState {
    let rules = UnoRuleSet {
        stack_pack: true,
        ..UnoRuleSet::default()
    };
    GameState::new_with_deck(rules, 6, build_deck_for_rules(rules)).unwrap()
}

#[test]
fn colored_stack_cards_require_the_current_color_and_accumulate() {
    let stack_one = card(UnoColor::Red, UnoFace::StackOne, 0);
    let matching_stack_two = card(UnoColor::Red, UnoFace::StackTwo, 0);
    let wrong_stack_two = card(UnoColor::Blue, UnoFace::StackTwo, 0);
    let filler = card(UnoColor::Green, UnoFace::Number(3), 0);
    let mut game = stack_pack_game();
    game.rules.action_stacking = true;
    game.players[0].hand = vec![stack_one, filler];
    game.players[1].hand = vec![matching_stack_two, wrong_stack_two, filler];
    game.discard_pile = vec![card(UnoColor::Red, UnoFace::Number(7), 0)];
    game.current_color = Some(UnoColor::Red);
    game.current_player = UnoPlayerId(0);

    game.play_card(UnoPlayerId(0), stack_one, None).unwrap();
    assert!(!game.can_play(UnoPlayerId(1), wrong_stack_two));
    assert!(game.can_play(UnoPlayerId(1), matching_stack_two));
    game.play_card(UnoPlayerId(1), matching_stack_two, None)
        .unwrap();

    let turn = game.turn().unwrap();
    assert_eq!(turn.pending_draw, 3);
    assert_eq!(turn.pending_kind, Some(UnoPendingDrawKind::Stack));
    assert_eq!(turn.pending_draw_source, Some(UnoPlayerId(1)));
    assert_eq!(turn.current_player, UnoPlayerId(2));
}

#[test]
fn same_stack_face_does_not_bypass_its_color_requirement() {
    let blue_stack = card(UnoColor::Blue, UnoFace::StackOne, 0);
    let filler = card(UnoColor::Green, UnoFace::Number(3), 0);
    let mut game = stack_pack_game();
    game.players[0].hand = vec![blue_stack, filler];
    game.discard_pile = vec![card(UnoColor::Red, UnoFace::StackOne, 0)];
    game.current_color = Some(UnoColor::Red);
    game.current_player = UnoPlayerId(0);

    assert!(!game.can_play(UnoPlayerId(0), blue_stack));
    assert_eq!(
        game.play_card(UnoPlayerId(0), blue_stack, None),
        Err(GameError::CardDoesNotMatch)
    );
}

#[test]
fn wild_stack_number_reveals_to_a_number_below_the_discard_top() {
    let stack_number = UnoCard::wild(UnoFace::WildStackNumber, 0);
    let revealed_action = card(UnoColor::Blue, UnoFace::Skip, 0);
    let revealed_number = card(UnoColor::Yellow, UnoFace::Number(6), 0);
    let old_top = card(UnoColor::Red, UnoFace::Number(7), 0);
    let filler = card(UnoColor::Green, UnoFace::Number(3), 0);
    let mut game = stack_pack_game();
    game.players[0].hand = vec![stack_number, filler];
    game.discard_pile = vec![old_top];
    game.draw_pile = VecDeque::from([revealed_action, revealed_number, filler]);
    game.current_color = Some(UnoColor::Red);
    game.current_player = UnoPlayerId(0);

    let outcome = game
        .play_card(UnoPlayerId(0), stack_number, Some(UnoColor::Blue))
        .unwrap();
    assert!(matches!(
        outcome,
        ActionOutcome::Played {
            effect: Some(PlayedEffect::StackNumberRevealed { ref cards, value: 6 }),
            ..
        } if cards == &[revealed_action, revealed_number]
    ));
    assert_eq!(
        game.discard_pile(),
        &[revealed_action, revealed_number, old_top, stack_number]
    );
    let turn = game.turn().unwrap();
    assert_eq!(turn.pending_draw, 6);
    assert_eq!(turn.pending_kind, Some(UnoPendingDrawKind::Stack));
    assert_eq!(turn.current_color, Some(UnoColor::Blue));
}

#[test]
fn stack_number_zero_is_still_an_active_penalty_turn() {
    let stack_number = UnoCard::wild(UnoFace::WildStackNumber, 0);
    let zero = card(UnoColor::Yellow, UnoFace::Number(0), 0);
    let filler = card(UnoColor::Green, UnoFace::Number(3), 0);
    let mut game = stack_pack_game();
    game.players[0].hand = vec![stack_number, filler];
    game.discard_pile = vec![card(UnoColor::Red, UnoFace::Number(7), 0)];
    game.draw_pile = VecDeque::from([zero, filler]);
    game.current_color = Some(UnoColor::Red);
    game.current_player = UnoPlayerId(0);

    game.play_card(UnoPlayerId(0), stack_number, Some(UnoColor::Yellow))
        .unwrap();
    assert_eq!(game.turn().unwrap().pending_draw, 0);
    assert_eq!(
        game.turn().unwrap().pending_kind,
        Some(UnoPendingDrawKind::Stack)
    );
    assert_eq!(
        game.draw_card(UnoPlayerId(1)),
        Err(GameError::MustResolveDrawPenalty)
    );
    assert!(matches!(
        game.accept_draw_penalty(UnoPlayerId(1)),
        Ok(ActionOutcome::PenaltyDrawn {
            ref cards,
            next_player: UnoPlayerId(2),
            ..
        }) if cards.is_empty()
    ));
}

#[test]
fn stack_cards_preserve_the_first_draw_four_challenge_offender() {
    let draw_four = UnoCard::wild(UnoFace::WildDrawFour, 0);
    let matching = card(UnoColor::Blue, UnoFace::Number(7), 0);
    let stack_one = card(UnoColor::Red, UnoFace::StackOne, 0);
    let filler = card(UnoColor::Green, UnoFace::Number(3), 0);
    let mut game = stack_pack_game();
    game.rules.action_stacking = true;
    game.players[0].hand = vec![draw_four, matching];
    game.players[1].hand = vec![stack_one, filler];
    game.discard_pile = vec![card(UnoColor::Blue, UnoFace::Number(5), 0)];
    game.current_color = Some(UnoColor::Blue);
    game.current_player = UnoPlayerId(0);

    game.play_card(UnoPlayerId(0), draw_four, Some(UnoColor::Red))
        .unwrap();
    game.play_card(UnoPlayerId(1), stack_one, None).unwrap();
    let outcome = game.challenge_draw_four(UnoPlayerId(2)).unwrap();
    assert!(matches!(
        outcome,
        ActionOutcome::ChallengeResolved {
            offender: UnoPlayerId(0),
            result: UnoChallengeResult::Successful,
            penalized: UnoPlayerId(0),
            ref cards,
            ..
        } if cards.len() == 5
    ));
}

#[test]
fn last_stack_card_finishes_after_applying_its_penalty() {
    let stack_two = card(UnoColor::Red, UnoFace::StackTwo, 0);
    let mut game = stack_pack_game();
    game.players[0].hand = vec![stack_two];
    game.discard_pile = vec![card(UnoColor::Red, UnoFace::Number(7), 0)];
    game.current_color = Some(UnoColor::Red);
    game.current_player = UnoPlayerId(0);

    assert!(matches!(
        game.play_card(UnoPlayerId(0), stack_two, None),
        Ok(ActionOutcome::Played { .. })
    ));
    assert_eq!(game.turn().unwrap().pending_draw, 2);
    let before = game.players[1].hand.len();
    assert!(matches!(
        game.accept_draw_penalty(UnoPlayerId(1)),
        Ok(ActionOutcome::PenaltyDrawn { ref cards, .. }) if cards.len() == 2
    ));
    assert_eq!(game.players[1].hand.len(), before + 2);
    assert!(matches!(
        game.phase(),
        Phase::Finished(GameResult {
            winner: UnoPlayerId(0),
            ..
        })
    ));
}
